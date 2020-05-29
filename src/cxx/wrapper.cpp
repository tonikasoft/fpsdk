#include "wrapper.h"
#include "fp_plugclass.h"
#include <cstring>
#include <stdlib.h>

intptr_t init_p_notes_params(int target, int flags, int ch_num, int pat_num,
                             TNoteParams *notes, int len) {
    TNotesParams *params = (TNotesParams *)malloc(sizeof(TNotesParams) +
                                                  sizeof(TNoteParams) * len);
    params->Target = target;
    params->Flags = flags;
    params->PatNum = pat_num;
    params->ChanNum = ch_num;
    params->Count = len;
    params->NoteParams[0] = *notes;
    memmove(params->NoteParams, notes, sizeof(TNoteParams) * len);

    return (intptr_t)params;
}

char *alloc_real_cstr(char *rust_cstr) {
    char *result = (char *)malloc(strlen(rust_cstr) + 1);
    strcpy(result, rust_cstr);
    free_rstring(rust_cstr);

    return result;
}

int istream_read(void *istream, unsigned char *data, unsigned int size,
                 unsigned int *read) {
    if (!data || size < 1)
        return 0x80004003; // E_POINTER

    return (int)((IStream *)istream)->Read(data, size, (unsigned long *)read);
}

int istream_write(void *istream, const unsigned char *data, unsigned int size,
                  unsigned int *write) {
    if (!data || size < 1)
        return 0x80004003; // E_POINTER

    return (int)((IStream *)istream)->Write(data, size, (unsigned long *)write);
}

void *create_plug_instance_c(void *host, intptr_t tag, void *adapter) {
    Info *info = plugin_info((PluginAdapter *)adapter);

    PFruityPlugInfo c_info = (TFruityPlugInfo *)malloc(sizeof(TFruityPlugInfo));
    c_info->SDKVersion = (int)info->sdk_version;
    c_info->LongName = info->long_name;
    c_info->ShortName = info->short_name;
    c_info->Flags = (int)info->flags;
    c_info->NumParams = (int)info->num_params;
    c_info->DefPoly = (int)info->def_poly;
    c_info->NumOutCtrls = (int)info->num_out_ctrls;
    c_info->NumOutVoices = (int)info->num_out_voices;

    free_rbox_raw(info);

    PluginWrapper *wrapper = new PluginWrapper(
        (TFruityPlugHost *)host, tag, (PluginAdapter *)adapter, c_info);

    return wrapper;
}

PluginWrapper::PluginWrapper(TFruityPlugHost *host_ptr, TPluginTag tag,
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
    free(Info);
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

    if (id == FPD_ShowEditor) {
        EditorHandle = (HWND)value;
    }

    FlMessage message = {id, index, value};

    return plugin_dispatcher(adapter, message);
}

void _stdcall PluginWrapper::GetName(int section, int index, int value,
                                     char *name) {
    FlMessage message = {
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
    FlMessage message = {
        (intptr_t)event_id,
        (intptr_t)event_value,
        (intptr_t)flags,
    };

    plugin_process_event(adapter, message);

    return 0;
}

int _stdcall PluginWrapper::ProcessParam(int index, int value, int rec_flags) {
    FlMessage message = {
        (intptr_t)index,
        (intptr_t)value,
        (intptr_t)rec_flags,
    };

    return (int)plugin_process_param(adapter, message);
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
    FlMessage message = {
        (intptr_t)event_id,
        (intptr_t)event_value,
        (intptr_t)flags,
    };

    return (int)voice_handler_on_event(adapter, (void *)handle, message);
}

int _stdcall PluginWrapper::Voice_Render(TVoiceHandle, PWAV32FS, int &) {
    // Deprecated:
    // https://forum.image-line.com/viewtopic.php?f=100&t=199515#p1371655
    return 0;
}

void _stdcall PluginWrapper::NewTick() { plugin_tick(adapter); }

void _stdcall PluginWrapper::MIDITick() { plugin_midi_tick(adapter); }

void _stdcall PluginWrapper::MIDIIn(int &msg) { plugin_midi_in(adapter, msg); }

void _stdcall PluginWrapper::MsgIn(intptr_t msg) {
    plugin_loop_in(adapter, msg);
}

int _stdcall PluginWrapper::OutputVoice_ProcessEvent(TOutVoiceHandle handle,
                                                     int event_id,
                                                     int event_value,
                                                     int flags) {
    FlMessage message = {
        (intptr_t)event_id,
        (intptr_t)event_value,
        (intptr_t)flags,
    };

    return (int)out_voice_handler_on_event(adapter, handle, message);
}

void _stdcall PluginWrapper::OutputVoice_Kill(TVoiceHandle handle) {
    out_voice_handler_kill(adapter, handle);
}

// host
intptr_t host_on_message(void *host, TPluginTag tag, FlMessage message) {
    return ((TFruityPlugHost *)host)
        ->Dispatcher(tag, message.id, message.index, message.value);
}

void host_on_parameter(void *host, TPluginTag tag, int index, int value) {
    ((TFruityPlugHost *)host)->OnParamChanged(tag, index, value);
}

void host_on_controller(void *host, TPluginTag tag, intptr_t index,
                        intptr_t value) {
    ((TFruityPlugHost *)host)->OnControllerChanged(tag, index, value);
}

void host_on_hint(void *host, TPluginTag tag, char *text) {
    ((TFruityPlugHost *)host)->OnHint(tag, text);
}

void host_midi_out(void *host, TPluginTag tag, unsigned char status,
                   unsigned char data1, unsigned char data2,
                   unsigned char port) {
    TMIDIOutMsg *msg = (TMIDIOutMsg *)malloc(sizeof(TMIDIOutMsg));
    msg->Status = status;
    msg->Data1 = data1;
    msg->Data2 = data2;
    msg->Port = port;

    ((TFruityPlugHost *)host)->MIDIOut(tag, (intptr_t)msg);
}

void host_midi_out_del(void *host, TPluginTag tag, unsigned char status,
                       unsigned char data1, unsigned char data2,
                       unsigned char port) {
    TMIDIOutMsg *msg = (TMIDIOutMsg *)malloc(sizeof(TMIDIOutMsg));
    msg->Status = status;
    msg->Data1 = data1;
    msg->Data2 = data2;
    msg->Port = port;

    ((TFruityPlugHost *)host)->MIDIOut_Delayed(tag, (intptr_t)msg);
}

void host_loop_out(void *host, TPluginTag tag, intptr_t msg) {
    ((TFruityPlugHost *)host)->PlugMsg_Delayed(tag, msg);
}

void host_loop_kill(void *host, TPluginTag tag, intptr_t msg) {
    ((TFruityPlugHost *)host)->PlugMsg_Kill(tag, msg);
}

void host_lock_mix(void *host) { ((TFruityPlugHost *)host)->LockMix(); }

void host_unlock_mix(void *host) { ((TFruityPlugHost *)host)->UnlockMix(); }

void host_lock_plugin(void *host, TPluginTag tag) {
    ((TFruityPlugHost *)host)->LockPlugin(tag);
}

void host_unlock_plugin(void *host, TPluginTag tag) {
    ((TFruityPlugHost *)host)->UnlockPlugin(tag);
}

void host_suspend_out(void *host) {
    ((TFruityPlugHost *)host)->SuspendOutput();
}

void host_resume_out(void *host) { ((TFruityPlugHost *)host)->ResumeOutput(); }

TIOBuffer host_get_input_buf(void *host, TPluginTag tag, intptr_t offset) {
    TIOBuffer buf = {
        0,
        0,
    };
    ((TFruityPlugHost *)host)->GetInBuffer(tag, offset, &buf);

    return buf;
}

TIOBuffer host_get_output_buf(void *host, TPluginTag tag, intptr_t offset) {
    TIOBuffer buf = {
        0,
        0,
    };
    ((TFruityPlugHost *)host)->GetOutBuffer(tag, offset, &buf);

    return buf;
}

void *host_get_insert_buf(void *host, TPluginTag tag, intptr_t offset) {
    return ((TFruityPlugHost *)host)->GetInsBuffer(tag, (int)offset);
}

void *host_get_mix_buf(void *host, intptr_t offset) {
    return ((TFruityPlugHost *)host)->GetMixBuffer((int)offset);
}

void *host_get_send_buf(void *host, intptr_t offset) {
    return ((TFruityPlugHost *)host)->GetSendBuffer(offset);
}

bool prompt_show(void *host, int x, int y, char *msg, char *result,
                 int &color) {

    return ((TFruityPlugHost *)host)->PromptEdit(x, y, msg, result, color);
}

// Host voice-related

intptr_t host_on_voice_event(void *host, intptr_t tag, FlMessage message) {
    return ((TFruityPlugHost *)host)
        ->Voice_ProcessEvent((TOutVoiceHandle)tag, message.id, message.index,
                             message.value);
}

void host_kill_voice(void *host, intptr_t tag) {
    ((TFruityPlugHost *)host)->Voice_Kill(tag, true);
}

void host_release_voice(void *host, intptr_t tag) {
    ((TFruityPlugHost *)host)->Voice_Release(tag);
}

intptr_t host_trig_out_voice(void *host, Params *params, int index,
                             intptr_t tag) {
    return (intptr_t)((TFruityPlugHost *)host)
        ->TriggerOutputVoice((TVoiceParams *)params, (intptr_t)index, tag);
}

void host_release_out_voice(void *host, intptr_t tag) {
    ((TFruityPlugHost *)host)->OutputVoice_Release((TOutVoiceHandle)tag);
}

void host_kill_out_voice(void *host, intptr_t tag) {
    ((TFruityPlugHost *)host)->OutputVoice_Kill((TOutVoiceHandle)tag);
}

intptr_t host_on_out_voice_event(void *host, intptr_t tag, FlMessage message) {
    return ((TFruityPlugHost *)host)
        ->OutputVoice_ProcessEvent((TOutVoiceHandle)tag, message.id,
                                   message.index, message.value);
}
