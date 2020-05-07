#include "wrapper.h"
#include "src/lib.rs.h"
#include <cstring>

TimeSignature time_sig_from_raw(intptr_t raw_time_sig) {
    PTimeSigInfo time_sig = (TTimeSigInfo *)raw_time_sig;

    return {(uint32_t)time_sig->StepsPerBar, (uint32_t)time_sig->StepsPerBeat,
            (uint32_t)time_sig->PPQ};
}

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

void *create_plug_instance_c(void *host, intptr_t tag, void *adapter) {
    Info *info = plugin_info((PluginAdapter *)adapter);

    int reserved[30] = {0};
    PFruityPlugInfo c_info = new TFruityPlugInfo{(int)info->sdk_version,
                                                 info->long_name,
                                                 info->short_name,
                                                 (int)info->flags,
                                                 (int)info->num_params,
                                                 (int)info->def_poly,
                                                 (int)info->num_out_ctrls,
                                                 (int)info->num_out_voices,
                                                 {*reserved}};

    free_rbox_raw(info);

    int ver = ((TFruityPlugHost *)host)->HostVersion;
    std::string sver = std::to_string(ver);
    fplog(rust::Str(sver.c_str()));
    fplog(rust::Str("host version above"));

    PluginWrapper *wrapper = new PluginWrapper(
        (TFruityPlugHost *)host, tag, (PluginAdapter *)adapter, c_info);

    return wrapper;
}

PluginWrapper::PluginWrapper(TFruityPlugHost *host_ptr, int tag,
                             PluginAdapter *adap, PFruityPlugInfo info) {
    Info = info;
    HostTag = tag;
    EditorHandle = 0;
    host = host_ptr;
    adapter = adap;
}

PluginWrapper::~PluginWrapper() {
    free(Info->LongName);
    free(Info->ShortName);
    delete Info;
    free_rbox_raw(adapter);
}

void _stdcall PluginWrapper::SaveRestoreState(IStream *stream, BOOL save) {
    if (save) {
        plugin_save_state(adapter, stream);
    } else {
        plugin_load_state(adapter, stream);
    }
}

intptr_t _stdcall PluginWrapper::Dispatcher(intptr_t id, intptr_t index,
                                            intptr_t value) {

    // if (ID == FPD_SetEnabled) {
    // host->Dispatcher(HostTag, FHD_WantMIDIInput, 0, Value);
    // }

    Message message = {id, index, value};

    return plugin_dispatcher(adapter, message);
}

void _stdcall PluginWrapper::GetName(int section, int index, int value,
                                     char *name) {
    Message message = {
        (intptr_t)section,
        (intptr_t)index,
        (intptr_t)value,
    };

    char *name_of = plugin_name_of(adapter, message);
    strcpy(name, name_of);
    free_rstring(name_of);
}

int _stdcall PluginWrapper::ProcessEvent(int event_id, int event_value,
                                         int flags) {
    Message message = {
        (intptr_t)event_id,
        (intptr_t)event_value,
        (intptr_t)flags,
    };

    plugin_process_event(adapter, message);

    return 0;
}

int _stdcall PluginWrapper::ProcessParam(int index, int value, int rec_flags) {
    Message message = {
        (intptr_t)index,
        (intptr_t)value,
        (intptr_t)rec_flags,
    };

    return plugin_process_param(adapter, message);
}

void _stdcall PluginWrapper::Idle_Public() { plugin_idle(adapter); }

void _stdcall PluginWrapper::Eff_Render(PWAV32FS source_buffer,
                                        PWAV32FS dest_buffer, int length) {
    plugin_eff_render(adapter, *source_buffer, *dest_buffer, length);
}

void _stdcall PluginWrapper::Gen_Render(PWAV32FS dest_buffer, int &length) {
    plugin_gen_render(adapter, *dest_buffer, length);
}

TVoiceHandle _stdcall PluginWrapper::TriggerVoice(PVoiceParams voice_params,
                                                  intptr_t set_tag) {
    LevelParams init_levels = {
        voice_params->InitLevels.Pan,   voice_params->InitLevels.Vol,
        voice_params->InitLevels.Pitch, voice_params->InitLevels.FCut,
        voice_params->InitLevels.FRes,
    };

    LevelParams final_levels = {
        voice_params->FinalLevels.Pan,   voice_params->FinalLevels.Vol,
        voice_params->FinalLevels.Pitch, voice_params->FinalLevels.FCut,
        voice_params->FinalLevels.FRes,
    };

    Params params = {
        init_levels,
        final_levels,
    };

    return (TVoiceHandle)voice_handler_trigger(adapter, params, set_tag);
}

void _stdcall PluginWrapper::Voice_Release(TVoiceHandle handle) {
    voice_handler_release(adapter, (void *)handle);
}

void _stdcall PluginWrapper::Voice_Kill(TVoiceHandle handle) {
    voice_handler_kill(adapter, (void *)handle);
}

int _stdcall PluginWrapper::Voice_ProcessEvent(TVoiceHandle handle,
                                               int event_id, int event_value,
                                               int flags) {
    Message message = {
        (intptr_t)event_id,
        (intptr_t)event_value,
        (intptr_t)flags,
    };

    return voice_handler_on_event(adapter, (void *)handle, message);
}

int _stdcall PluginWrapper::Voice_Render(TVoiceHandle, PWAV32FS, int &) {
    // Deprecated:
    // https://forum.image-line.com/viewtopic.php?f=100&t=199515#p1371655
    return 0;
}

void _stdcall PluginWrapper::NewTick() { plugin_tick(adapter); }

void _stdcall PluginWrapper::MIDITick() { plugin_midi_tick(adapter); }

void _stdcall PluginWrapper::MIDIIn(int &msg) {
    MidiMessage message = {
        (uint8_t)(msg & 0xff),
        (uint8_t)((msg >> 8) & 0xff),
        (uint8_t)((msg >> 16) & 0xff),
        (int)((msg >> 24) & 0xff),
    };
    plugin_midi_in(adapter, message);
}

void _stdcall PluginWrapper::MsgIn(intptr_t msg) {}

int _stdcall PluginWrapper::OutputVoice_ProcessEvent(TOutVoiceHandle handle,
                                                     int event_id,
                                                     int event_value,
                                                     int flags) {
    Message message = {
        (intptr_t)event_id,
        (intptr_t)event_value,
        (intptr_t)flags,
    };

    return out_voice_handler_on_event(adapter, handle, message);
}

void _stdcall PluginWrapper::OutputVoice_Kill(TVoiceHandle handle) {
    out_voice_handler_kill(adapter, handle);
}

void host_release_voice(void *host, intptr_t tag) {
    ((TFruityPlugHost *)host)->Voice_Release(tag);
}

void host_kill_voice(void *host, intptr_t tag) {
    ((TFruityPlugHost *)host)->Voice_Kill(tag, true);
}

intptr_t host_on_voice_event(void *host, intptr_t tag, Message message) {
    return ((TFruityPlugHost *)host)
        ->Voice_ProcessEvent((TOutVoiceHandle)tag, message.id, message.index,
                             message.value);
}

intptr_t host_trig_out_voice(void *host, Params *params, int32_t index,
                             intptr_t tag) {
    return (intptr_t)((TFruityPlugHost *)host)
        ->TriggerOutputVoice((TVoiceParams *)params, index, tag);
}

void host_release_out_voice(void *host, intptr_t tag) {
    ((TFruityPlugHost *)host)->OutputVoice_Release((TOutVoiceHandle)tag);
}

void host_kill_out_voice(void *host, intptr_t tag) {
    ((TFruityPlugHost *)host)->OutputVoice_Kill((TOutVoiceHandle)tag);
}

intptr_t host_on_out_voice_event(void *host, intptr_t tag, Message message) {
    return ((TFruityPlugHost *)host)
        ->OutputVoice_ProcessEvent((TOutVoiceHandle)tag, message.id,
                                   message.index, message.value);
}
