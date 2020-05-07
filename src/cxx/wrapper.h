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
    PluginWrapper(TFruityPlugHost *Host, int Tag, PluginAdapter *adapter,
                  PFruityPlugInfo info);
    virtual ~PluginWrapper();

    // from TFruityPlug
    virtual intptr_t _stdcall Dispatcher(intptr_t ID, intptr_t Index,
                                         intptr_t Value);
    virtual void _stdcall Idle_Public();
    virtual void _stdcall SaveRestoreState(IStream *Stream, BOOL Save);
    virtual void _stdcall GetName(int Section, int Index, int Value,
                                  char *Name);
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

// Voice handler
extern "C" intptr_t voice_handler_trigger(PluginAdapter *adpater, Params params,
                                          intptr_t tag);
extern "C" void voice_handler_release(PluginAdapter *adpater, void *voice);
extern "C" void voice_handler_kill(PluginAdapter *adpater, void *voice);
extern "C" intptr_t voice_handler_on_event(PluginAdapter *adpater, void *voice,
                                           Message message);
extern "C" void out_voice_handler_kill(PluginAdapter *adpater, intptr_t tag);
extern "C" intptr_t out_voice_handler_on_event(PluginAdapter *adpater,
                                               intptr_t tag, Message message);
// IStream
extern "C" int32_t istream_read(void *istream, uint8_t *data, uint32_t size,
                                uint32_t *read);
extern "C" int32_t istream_write(void *istream, const uint8_t *data,
                                 uint32_t size, uint32_t *write);
// Host
extern "C" intptr_t host_trig_out_voice(void *host, Params *params,
                                        int32_t index, intptr_t tag);
extern "C" void host_release_out_voice(void *host, intptr_t tag);
extern "C" void host_kill_out_voice(void *host, intptr_t tag);
extern "C" intptr_t host_on_out_voice_event(void *host, intptr_t tag,
                                            Message message);

// Utility
extern "C" void free_rbox_raw(void *raw_ptr);
extern "C" void free_rstring(char *raw_str);
// FFI to make C string (`char *`) managed by C side. Because `char *` produced
// by `CString::into_raw` leads to memory leak. Here's what docs say about
// `CString::into_raw`:
//
// The pointer which this function returns must be returned to Rust and
// reconstituted using from_raw to be properly deallocated. Specifically, one
// should not use the standard C free() function to deallocate this string.
extern "C" char *alloc_real_cstr(char *rust_cstr);
