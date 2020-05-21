#pragma once

#include "fp_plugclass.h"
#include "rust/cxx.h"

struct Message;
struct MidiMessage;
struct PluginAdapter;
struct TimeSignature;

// from plugin.rs
struct Info {
    uint32_t sdk_version;
    char *long_name;
    char *short_name;
    uint32_t flags;
    uint32_t num_params;
    uint32_t def_poly;
    uint32_t num_out_ctrls;
    uint32_t num_out_voices;
};

// from voice.rs
struct LevelParams {
    float pan;
    float vol;
    float pitch;
    float mod_x;
    float mod_y;
};

struct Params {
    LevelParams init_levels;
    LevelParams final_levels;
};

class PluginWrapper : public TFruityPlug {
  public:
    PluginWrapper(TFruityPlugHost *host, TPluginTag tag, PluginAdapter *adapter,
                  PFruityPlugInfo info);
    virtual ~PluginWrapper();

    // from TFruityPlug
    virtual intptr_t _stdcall Dispatcher(intptr_t id, intptr_t index,
                                         intptr_t value);
    virtual void _stdcall Idle_Public();
    virtual void _stdcall SaveRestoreState(IStream *stream, BOOL save);
    virtual void _stdcall GetName(int section, int index, int value,
                                  char *name);
    virtual int _stdcall ProcessEvent(int event_id, int event_value, int flags);
    virtual int _stdcall ProcessParam(int index, int value, int rec_flags);
    virtual void _stdcall Eff_Render(PWAV32FS source_buffer,
                                     PWAV32FS dest_buffer, int length);
    virtual void _stdcall Gen_Render(PWAV32FS dest_buffer, int &length);
    virtual TVoiceHandle _stdcall TriggerVoice(PVoiceParams voice_params,
                                               intptr_t set_tag);
    virtual void _stdcall Voice_Release(TVoiceHandle handle);
    virtual void _stdcall Voice_Kill(TVoiceHandle handle);
    virtual int _stdcall Voice_ProcessEvent(TVoiceHandle handle, int event_id,
                                            int event_value, int flags);
    virtual int _stdcall Voice_Render(TVoiceHandle handle, PWAV32FS dest_buffer,
                                      int &length);
    virtual void _stdcall NewTick();
    virtual void _stdcall MIDITick();
    virtual void _stdcall MIDIIn(int &msg);
    virtual void _stdcall MsgIn(intptr_t msg);
    virtual int _stdcall OutputVoice_ProcessEvent(TOutVoiceHandle handle,
                                                  int event_id, int event_value,
                                                  int flags);
    virtual void _stdcall OutputVoice_Kill(TVoiceHandle handle);

  protected:
    TFruityPlugHost *host;
    PluginAdapter *adapter;
};

TimeSignature time_sig_from_raw(intptr_t raw_time_sig);

// Unsafe Rust FFI
//
// PluginAdapter methods
extern "C" void *create_plug_instance_c(void *host, intptr_t tag,
                                        void *adapter);
extern "C" Info *plugin_info(PluginAdapter *adapter);
extern "C" intptr_t plugin_dispatcher(PluginAdapter *adapter, Message message);
extern "C" intptr_t plugin_process_event(PluginAdapter *adapter, Message event);
extern "C" intptr_t plugin_process_param(PluginAdapter *adapter, Message event);
extern "C" char *plugin_name_of(const PluginAdapter *adapter, Message message);
extern "C" void plugin_idle(PluginAdapter *adapter);
extern "C" void plugin_tick(PluginAdapter *adapter);
extern "C" void plugin_midi_tick(PluginAdapter *adapter);
extern "C" void plugin_eff_render(PluginAdapter *adapter,
                                  const float source[1][2], float dest[1][2],
                                  int len);
extern "C" void plugin_gen_render(PluginAdapter *adapter, float dest[1][2],
                                  int len);
extern "C" void plugin_midi_in(PluginAdapter *adapter, MidiMessage message);
extern "C" void plugin_save_state(PluginAdapter *adapter, IStream *istream);
extern "C" void plugin_load_state(PluginAdapter *adapter, IStream *istream);
extern "C" void plugin_loop_in(PluginAdapter *adapter, intptr_t message);

// Voice handler
extern "C" intptr_t voice_handler_trigger(PluginAdapter *adapter, Params params,
                                          intptr_t tag);
extern "C" void voice_handler_release(PluginAdapter *adapter, void *voice);
extern "C" void voice_handler_kill(PluginAdapter *adapter, void *voice);
extern "C" intptr_t voice_handler_on_event(PluginAdapter *adapter, void *voice,
                                           Message message);
extern "C" void out_voice_handler_kill(PluginAdapter *adapter, intptr_t tag);
extern "C" intptr_t out_voice_handler_on_event(PluginAdapter *adapter,
                                               intptr_t tag, Message message);

// IStream
extern "C" int32_t istream_read(void *istream, uint8_t *data, uint32_t size,
                                uint32_t *read);
extern "C" int32_t istream_write(void *istream, const uint8_t *data,
                                 uint32_t size, uint32_t *write);

// Host
extern "C" intptr_t host_on_message(void *host, TPluginTag tag,
                                    Message message);
extern "C" void host_on_parameter(void *host, TPluginTag tag, int index,
                                  int value);
extern "C" void host_on_controller(void *host, TPluginTag tag, intptr_t index,
                                   intptr_t value);
extern "C" void host_on_hint(void *host, TPluginTag tag, char *text);
extern "C" void host_midi_out(void *host, TPluginTag tag, unsigned char status,
                              unsigned char data1, unsigned char data2,
                              unsigned char port);
extern "C" void host_midi_out_del(void *host, TPluginTag tag,
                                  unsigned char status, unsigned char data1,
                                  unsigned char data2, unsigned char port);
extern "C" void host_loop_out(void *host, TPluginTag tag, intptr_t msg);
extern "C" void host_loop_kill(void *host, TPluginTag tag, intptr_t msg);
extern "C" void host_lock_mix(void *host);
extern "C" void host_unlock_mix(void *host);
extern "C" void host_lock_plugin(void *host, TPluginTag tag);
extern "C" void host_unlock_plugin(void *host, TPluginTag tag);
extern "C" void host_suspend_out(void *host);
extern "C" void host_resume_out(void *host);
extern "C" TIOBuffer host_get_input_buf(void *host, TPluginTag tag,
                                        intptr_t offset);
extern "C" TIOBuffer host_get_output_buf(void *host, TPluginTag tag,
                                         intptr_t offset);
extern "C" void *host_get_insert_buf(void *host, TPluginTag tag,
                                     intptr_t offset);
extern "C" void *host_get_mix_buf(void *host, intptr_t offset);
extern "C" void *host_get_send_buf(void *host, intptr_t offset);

extern "C" bool prompt_show(void *host, int x, int y, char *msg, char *result,
                            int &color);

// Host voice-related
extern "C" void host_release_voice(void *host, intptr_t tag);
extern "C" void host_kill_voice(void *host, intptr_t tag);
extern "C" intptr_t host_on_voice_event(void *host, intptr_t tag,
                                        Message message);
extern "C" intptr_t host_trig_out_voice(void *host, Params *params,
                                        int32_t index, intptr_t tag);
extern "C" void host_release_out_voice(void *host, intptr_t tag);
extern "C" void host_kill_out_voice(void *host, intptr_t tag);
extern "C" intptr_t host_on_out_voice_event(void *host, intptr_t tag,
                                            Message message);

// Utility
extern "C" intptr_t init_p_notes_params(int target, int flags, int ch_num,
                                        int pat_num, TNoteParams *notes,
                                        int len);

extern "C" void free_rbox_raw(void *raw_ptr);
extern "C" void free_rstring(char *raw_str);
// FFI to make C string (`char *`) managed by C side. Because `char *`
// produced by `CString::into_raw` leads to memory leak. Here's what docs
// say about `CString::into_raw`:
//
// The pointer which this function returns must be returned to Rust and
// reconstituted using from_raw to be properly deallocated. Specifically,
// one should not use the standard C free() function to deallocate this
// string.
extern "C" char *alloc_real_cstr(char *rust_cstr);
