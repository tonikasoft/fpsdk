#include "wrapper.h"
#include "src/lib.rs.h"
#include <cstring>

char *init_str_from_rust(rust::String &value) {
  char *res = new char[value.size()];
  strcpy(res, value.data());
  res[value.size()] = 0x0000;

  return res;
}

TFruityPlug &create_plug_instance_c(TFruityPlugHost &Host, int Tag,
                                    rust::Box<PluginAdapter> adapter) {
  Info info = plugin_info(*adapter);

  char *lname = init_str_from_rust(info.long_name);
  char *sname = init_str_from_rust(info.short_name);

  PFruityPlugInfo c_info = new TFruityPlugInfo{(int)info.sdk_version,
                                               lname,
                                               sname,
                                               (int)info.flags,
                                               (int)info.num_params,
                                               (int)info.def_poly,
                                               (int)info.num_out_ctrls,
                                               (int)info.num_out_voices};

  PluginWrapper *wrapper =
      new PluginWrapper(&Host, Tag, adapter.into_raw(), c_info);

  return *((TFruityPlug *)wrapper);
}

PluginWrapper::PluginWrapper(TFruityPlugHost *Host, int Tag,
                             PluginAdapter *adap, PFruityPlugInfo info) {
  Info = info;
  HostTag = Tag;
  EditorHandle = 0;
  _host = Host;
  _editor = nullptr;
  adapter = adap;

  // parameter initialze
  _gain = 0.25;
  _params[0] = (1 << 16);
}

PluginWrapper::~PluginWrapper() {
  delete _editor;
  delete Info->LongName;
  delete Info->ShortName;
  free(Info);
  free(adapter);
}

void _stdcall PluginWrapper::SaveRestoreState(IStream *Stream, BOOL Save) {
  if (Save) {
    // save paremeters
    unsigned long length = 0;
    Stream->Write(_params, sizeof(_params), &length);
  } else {
    // load paremeters
    unsigned long length = 0;
    Stream->Read(_params, sizeof(_params), &length);
    for (int ii = 0; ii < Info->NumParams; ii++) {
      if (ii == 0) {
        _gain = static_cast<float>(_params[ii]) / (1 << 16);
      }

      // if( _editor != nullptr )
      // {
      // // send message to editor
      // _editor->setParameter(ii, static_cast<float>(_params[ii]));
      // }
    }
  }
}

intptr_t _stdcall PluginWrapper::Dispatcher(intptr_t ID, intptr_t Index,
                                            intptr_t Value) {
  Message message = {ID, Index, Value};

  return plugin_dispatcher(adapter, message);
}

void _stdcall PluginWrapper::GetName(int Section, int Index, int Value,
                                     char *Name) {
  Message message = {
      (intptr_t)Section,
      (intptr_t)Index,
      (intptr_t)Value,
  };

  strcpy(Name, plugin_name_of(*adapter, message).data());
}

int _stdcall PluginWrapper::ProcessEvent(int EventID, int EventValue,
                                         int Flags) {
  Message message = {
      (intptr_t)EventID,
      (intptr_t)EventValue,
      (intptr_t)Flags,
  };

  plugin_process_event(adapter, message);

  return 0;
}

//----------------
//
//----------------
int _stdcall PluginWrapper::ProcessParam(int Index, int Value, int RECFlags) {
  int ret = 0;
  if (Index < Info->NumParams) {
    if (RECFlags & REC_UpdateValue) {
      _params[Index] = Value;

      char hinttext[256] = {0};
      if (Index == 0) {
        _gain = static_cast<float>(Value) / (1 << 16);
        if (_gain < 1.0e-8) {
          // convert to dB
          // sprintf_s(hinttext, "Gain: -oo dB");
        } else {
          // convert to dB
          // sprintf_s(hinttext, "Gain: %.3f dB", 20.0 * log10(_gain));
        }
      }

      // display text to hint bar
      _host->OnHint(Index, hinttext);

      if (RECFlags & REC_UpdateControl) {
        // send message to editor
        // _editor->setParameter(Index, static_cast<float>(Value));
      } else {
        // send message to host
        _host->OnParamChanged(this->HostTag, Index, Value);
      }
    } else if (RECFlags & REC_GetValue) {
      // get parameter
      ret = _params[Index];
    }
  }
  return ret;
}

//----------------
// idle
//----------------
void _stdcall PluginWrapper::Idle_Public() {
  // if (_editor) _editor->doIdleStuff();
}

//----------------
// effect
//----------------
void _stdcall PluginWrapper::Eff_Render(PWAV32FS SourceBuffer,
                                        PWAV32FS DestBuffer, int Length) {
  float gain = _gain;
  for (int ii = 0; ii < Length; ii++) {
    (*DestBuffer)[ii][0] = (*SourceBuffer)[ii][0] * gain;
    (*DestBuffer)[ii][1] = (*SourceBuffer)[ii][1] * gain;
  }
}

void _stdcall PluginWrapper::Gen_Render(PWAV32FS DestBuffer, int &Length) {}

TVoiceHandle _stdcall PluginWrapper::TriggerVoice(PVoiceParams VoiceParams,
                                                  intptr_t SetTag) {
  return TVoiceHandle();
}

void _stdcall PluginWrapper::Voice_Release(TVoiceHandle Handle) {}

void _stdcall PluginWrapper::Voice_Kill(TVoiceHandle Handle) {}

int _stdcall PluginWrapper::Voice_ProcessEvent(TVoiceHandle Handle, int EventID,
                                               int EventValue, int Flags) {
  return 0;
}

int _stdcall PluginWrapper::Voice_Render(TVoiceHandle Handle,
                                         PWAV32FS DestBuffer, int &Length) {
  return 0;
}

void _stdcall PluginWrapper::NewTick() { plugin_tick(adapter); }

void _stdcall PluginWrapper::MIDITick() {}

void _stdcall PluginWrapper::MIDIIn(int &Msg) {}

void _stdcall PluginWrapper::MsgIn(intptr_t Msg) {}

int _stdcall PluginWrapper::OutputVoice_ProcessEvent(TOutVoiceHandle Handle,
                                                     int EventID,
                                                     int EventValue,
                                                     int Flags) {
  return 0;
}

void _stdcall PluginWrapper::OutputVoice_Kill(TVoiceHandle Handle) {}

TimeSignature time_sig_from_raw(intptr_t raw_time_sig) {
  PTimeSigInfo time_sig = (TTimeSigInfo *)raw_time_sig;

  return TimeSignature{(uint32_t)time_sig->StepsPerBar,
                       (uint32_t)time_sig->StepsPerBeat,
                       (uint32_t)time_sig->PPQ};
}
