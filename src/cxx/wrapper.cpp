#include "wrapper.h"
#include "src/lib.rs.h"
#include <cstring>

char *alloc_real_cstr(char *rust_cstr) {
    char *result = (char *)malloc(strlen(rust_cstr) + 1);
    strcpy(result, rust_cstr);
    free_rstring(rust_cstr);

    return result;
}

int32_t istream_read(void *istream, uint8_t *data, uint32_t size,
                     uint32_t *read) {
    if (!data || size < 1)
        return 0x80004003; // E_POINTER

    return (int32_t)((IStream *)istream)
        ->Read(data, size, (unsigned long *)read);
}

int32_t istream_write(void *istream, const uint8_t *data, uint32_t size,
                      uint32_t *write) {
    if (!data || size < 1)
        return 0x80004003; // E_POINTER

    return (int32_t)((IStream *)istream)
        ->Write(data, size, (unsigned long *)write);
}

void *create_plug_instance_c(void *Host, int Tag, void *adapter) {
    Info *info = plugin_info((PluginAdapter *)adapter);

    PFruityPlugInfo c_info = new TFruityPlugInfo{
        (int)info->sdk_version,   info->long_name,          info->short_name,
        (int)info->flags,         (int)info->num_params,    (int)info->def_poly,
        (int)info->num_out_ctrls, (int)info->num_out_voices};

    free_rbox_raw(info);

    int32_t ver = ((TFruityPlugHost *)Host)->HostVersion;
    std::string sver = std::to_string(ver);
    fplog(rust::Str(sver.c_str()));
    fplog(rust::Str("host version above"));

    PluginWrapper *wrapper = new PluginWrapper(
        (TFruityPlugHost *)Host, Tag, (PluginAdapter *)adapter, c_info);

    return wrapper;
}

PluginWrapper::PluginWrapper(TFruityPlugHost *Host, int Tag,
                             PluginAdapter *adap, PFruityPlugInfo info) {
    Info = info;
    HostTag = Tag;
    EditorHandle = 0;
    host = Host;
    adapter = adap;
}

PluginWrapper::~PluginWrapper() {
    free(Info->LongName);
    free(Info->ShortName);
    delete Info;
    free_rbox_raw(adapter);
}

void _stdcall PluginWrapper::SaveRestoreState(IStream *Stream, BOOL Save) {
    if (Save) {
        plugin_save_state(adapter, Stream);
    } else {
        plugin_load_state(adapter, Stream);
    }
}

intptr_t _stdcall PluginWrapper::Dispatcher(intptr_t ID, intptr_t Index,
                                            intptr_t Value) {

    // if (ID == FPD_SetEnabled) {
    // host->Dispatcher(HostTag, FHD_WantMIDIInput, 0, Value);
    // }

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

    char *name = plugin_name_of(adapter, message);
    strcpy(Name, name);
    free_rstring(name);
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

int _stdcall PluginWrapper::ProcessParam(int Index, int Value, int RECFlags) {
    Message message = {
        (intptr_t)Index,
        (intptr_t)Value,
        (intptr_t)RECFlags,
    };

    return plugin_process_param(adapter, message);
}

void _stdcall PluginWrapper::Idle_Public() { plugin_idle(adapter); }

void _stdcall PluginWrapper::Eff_Render(PWAV32FS SourceBuffer,
                                        PWAV32FS DestBuffer, int Length) {
    plugin_eff_render(adapter, *SourceBuffer, *DestBuffer, Length);
}

void _stdcall PluginWrapper::Gen_Render(PWAV32FS DestBuffer, int &Length) {
    plugin_gen_render(adapter, *DestBuffer, Length);
}

TVoiceHandle _stdcall PluginWrapper::TriggerVoice(PVoiceParams VoiceParams,
                                                  intptr_t SetTag) {
    LevelParams init_levels = {
        VoiceParams->InitLevels.Pan,   VoiceParams->InitLevels.Vol,
        VoiceParams->InitLevels.Pitch, VoiceParams->InitLevels.FCut,
        VoiceParams->InitLevels.FRes,
    };

    LevelParams final_levels = {
        VoiceParams->FinalLevels.Pan,   VoiceParams->FinalLevels.Vol,
        VoiceParams->FinalLevels.Pitch, VoiceParams->FinalLevels.FCut,
        VoiceParams->FinalLevels.FRes,
    };

    Params params = {
        init_levels,
        final_levels,
    };

    return (TVoiceHandle)voice_handler_trigger(adapter, params, (int)SetTag);
}

void _stdcall PluginWrapper::Voice_Release(TVoiceHandle Handle) {
    voice_handler_release(adapter, (void *)Handle);
}

void _stdcall PluginWrapper::Voice_Kill(TVoiceHandle Handle) {
    voice_handler_kill(adapter, (void *)Handle);
}

int _stdcall PluginWrapper::Voice_ProcessEvent(TVoiceHandle Handle, int EventID,
                                               int EventValue, int Flags) {
    Message message = {
        (intptr_t)EventID,
        (intptr_t)EventValue,
        (intptr_t)Flags,
    };

    return voice_handler_on_event(adapter, (void *)Handle, message);
}

int _stdcall PluginWrapper::Voice_Render(TVoiceHandle, PWAV32FS, int &) {
    // Deprecated:
    // https://forum.image-line.com/viewtopic.php?f=100&t=199515#p1371655
    return 0;
}

void _stdcall PluginWrapper::NewTick() { plugin_tick(adapter); }

void _stdcall PluginWrapper::MIDITick() { plugin_midi_tick(adapter); }

void _stdcall PluginWrapper::MIDIIn(int &Msg) {
    MidiMessage message = {
        (uint8_t)(Msg & 0xff),
        (uint8_t)((Msg >> 8) & 0xff),
        (uint8_t)((Msg >> 16) & 0xff),
        (int)((Msg >> 24) & 0xff),
    };
    plugin_midi_in(adapter, message);
}

void _stdcall PluginWrapper::MsgIn(intptr_t Msg) {}

int _stdcall PluginWrapper::OutputVoice_ProcessEvent(TOutVoiceHandle Handle,
                                                     int EventID,
                                                     int EventValue,
                                                     int Flags) {
    Message message = {
        (intptr_t)EventID,
        (intptr_t)EventValue,
        (intptr_t)Flags,
    };

    return out_voice_handler_on_event(adapter, (void *)Handle, message);
}

void _stdcall PluginWrapper::OutputVoice_Kill(TVoiceHandle Handle) {
    out_voice_handler_kill(adapter, (void *)Handle);
}

TimeSignature time_sig_from_raw(intptr_t raw_time_sig) {
    PTimeSigInfo time_sig = (TTimeSigInfo *)raw_time_sig;

    return {(uint32_t)time_sig->StepsPerBar, (uint32_t)time_sig->StepsPerBeat,
            (uint32_t)time_sig->PPQ};
}
