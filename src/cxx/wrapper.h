#pragma once

#include "rust/cxx.h"
#include "fp_plugclass.h"

struct PluginAdapter;

class sample_editor {};

class PluginWrapper : public TFruityPlug {
public:
  PluginWrapper(TFruityPlugHost *Host, int Tag, PluginAdapter &adapter,
                PFruityPlugInfo info);
  virtual ~PluginWrapper();

  // from TFruityPlug
  virtual intptr_t _stdcall Dispatcher(intptr_t ID, intptr_t Index,
                                       intptr_t Value);
  virtual void _stdcall Idle_Public();
  virtual void _stdcall SaveRestoreState(IStream *Stream, BOOL Save);
  virtual void _stdcall GetName(int Section, int Index, int Value, char *Name);
  virtual int _stdcall ProcessEvent(int EventID, int EventValue, int Flags);
  virtual int _stdcall ProcessParam(int Index, int Value, int RECFlags);
  virtual void _stdcall Eff_Render(PWAV32FS SourceBuffer, PWAV32FS DestBuffer,
                                   int Length);
  virtual void _stdcall Gen_Render(PWAV32FS DestBuffer, int &Length);
  virtual TVoiceHandle _stdcall TriggerVoice(PVoiceParams VoiceParams,
                                             intptr_t SetTag);
  virtual void _stdcall Voice_Release(TVoiceHandle Handle);
  virtual void _stdcall Voice_Kill(TVoiceHandle Handle);
  virtual int _stdcall Voice_ProcessEvent(TVoiceHandle Handle, int EventID,
                                          int EventValue, int Flags);
  virtual int _stdcall Voice_Render(TVoiceHandle Handle, PWAV32FS DestBuffer,
                                    int &Length);
  virtual void _stdcall NewTick();
  virtual void _stdcall MIDITick();
  virtual void _stdcall MIDIIn(int &Msg);
  virtual void _stdcall MsgIn(intptr_t Msg);
  virtual int _stdcall OutputVoice_ProcessEvent(TOutVoiceHandle Handle,
                                                int EventID, int EventValue,
                                                int Flags);
  virtual void _stdcall OutputVoice_Kill(TVoiceHandle Handle);

protected:
  // GUI
  sample_editor *_editor;

  // host
  TFruityPlugHost *_host;

  PluginAdapter *adapter;

  // parameter
  int _params[1024];

  // gain
  float _gain;
};

TFruityPlug &create_plug_instance_c(TFruityPlugHost &Host, int Tag,
                                    rust::Box<PluginAdapter> adapter);
