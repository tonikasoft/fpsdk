#include <cstring>
#include "wrapper.h"

//---------------------
// Plug-in information
//---------------------
char dllname[] = "Pocrs";
char name[] = "Pocrs";
TFruityPlugInfo PlugInfo =
{
	CurrentSDKVersion,
	dllname,
	name,
	FPF_Type_Effect,
	1 // the amount of parameters
};

TFruityPlug& create_plug_instance_c(TFruityPlugHost& Host, int64_t Tag) {
    Wrapper* wrapper = new Wrapper(&Host, (int) Tag);
    return *((TFruityPlug*) wrapper);
}

//----------------
// constructor
//----------------
Wrapper::Wrapper(TFruityPlugHost *Host, int Tag)
{
	Info = &PlugInfo;
	HostTag = Tag;
	EditorHandle = 0;
	_host = Host;
	_editor = nullptr;

	// parameter initialze
	_gain = 0.25;
	_params[0] = (1<<16);
}

//----------------
// destructor
//----------------
Wrapper::~Wrapper()
{
	delete _editor;
}

//-------------------------
// save or load parameter
//-------------------------
void _stdcall Wrapper::SaveRestoreState(IStream *Stream, BOOL Save)
{
	if( Save )
	{
		// save paremeters
		unsigned long length = 0;
		Stream->Write(_params, sizeof(_params), &length);
	}
	else
	{
		// load paremeters
		unsigned long length = 0;
		Stream->Read(_params, sizeof(_params), &length);
		for( int ii = 0; ii < NumParams; ii++ )
		{
			if( ii == 0 )
			{
				_gain = static_cast<float>(_params[ii]) / (1<<16);
			}

			// if( _editor != nullptr )
			// {
				// // send message to editor
				// _editor->setParameter(ii, static_cast<float>(_params[ii]));
			// }
		}
	}
}

//----------------
// 
//----------------
intptr_t _stdcall Wrapper::Dispatcher(intptr_t ID, intptr_t Index, intptr_t Value)
{
	// if( ID == FPD_ShowEditor )
	// {
		// if (Value == 0)
		// {
			// // close editor
			// delete _editor;
			// _editor = nullptr;
			// EditorHandle = 0;
		// }
		// else if( EditorHandle == 0 )
		// {
			// if (_editor == nullptr)
			// {
				// // first
				// _editor = new sample_editor(this, reinterpret_cast<HWND>(Value));
			// }
// 
			// // open editor
			// EditorHandle = reinterpret_cast<HWND>(_editor->getHWND());
		// }
		// else
		// {
			// // change parent window ?
			// ::SetParent(EditorHandle, reinterpret_cast<HWND>(Value));
		// }
	// }
	return 0;
}

//----------------
// 
//----------------
void _stdcall Wrapper::GetName(int Section, int Index, int Value, char *Name)
{
	if(Section == FPN_Param)
	{
        strcpy(Name, "Gain");
	}
}

int _stdcall Wrapper::ProcessEvent(int EventID, int EventValue, int Flags)
{
	return 0;
}

//----------------
// 
//----------------
int _stdcall Wrapper::ProcessParam(int Index, int Value, int RECFlags)
{
	int ret = 0;
	if( Index < NumParams )
	{
		if( RECFlags & REC_UpdateValue )
		{
			_params[Index] = Value;

			char hinttext[256] = { 0 };
			if( Index == 0 )
			{
				_gain = static_cast<float>(Value) / (1<<16);
				if( _gain < 1.0e-8)
				{
					// convert to dB
					// sprintf_s(hinttext, "Gain: -oo dB");
				}
				else
				{
					// convert to dB
					// sprintf_s(hinttext, "Gain: %.3f dB", 20.0 * log10(_gain));
				}
			}

			// display text to hint bar
			_host->OnHint(Index, hinttext);

			if( RECFlags & REC_UpdateControl )
			{
				// send message to editor
				// _editor->setParameter(Index, static_cast<float>(Value));
			}
			else
			{
				// send message to host
				_host->OnParamChanged(this->HostTag, Index, Value);
			}
		}
		else if( RECFlags & REC_GetValue )
		{
			// get parameter
			ret = _params[Index];
		}
	}
	return ret;
}

//----------------
// idle
//----------------
void _stdcall Wrapper::Idle_Public()
{
	// if (_editor) _editor->doIdleStuff();
}

//----------------
// effect
//----------------
void _stdcall Wrapper::Eff_Render(PWAV32FS SourceBuffer, PWAV32FS DestBuffer, int Length)
{
	float gain = _gain;
	for (int ii = 0; ii < Length; ii++)
	{
		(*DestBuffer)[ii][0] = (*SourceBuffer)[ii][0] * gain;
		(*DestBuffer)[ii][1] = (*SourceBuffer)[ii][1] * gain;
	}
}

void _stdcall Wrapper::Gen_Render(PWAV32FS DestBuffer, int& Length)
{
}

TVoiceHandle _stdcall Wrapper::TriggerVoice(PVoiceParams VoiceParams, intptr_t SetTag)
{
	return TVoiceHandle();
}

void _stdcall Wrapper::Voice_Release(TVoiceHandle Handle)
{
}

void _stdcall Wrapper::Voice_Kill(TVoiceHandle Handle)
{
}

int _stdcall Wrapper::Voice_ProcessEvent(TVoiceHandle Handle, int EventID, int EventValue, int Flags)
{
	return 0;
}

int _stdcall Wrapper::Voice_Render(TVoiceHandle Handle, PWAV32FS DestBuffer, int& Length)
{
	return 0;
}

void _stdcall Wrapper::NewTick()
{
}

void _stdcall Wrapper::MIDITick()
{
}

void _stdcall Wrapper::MIDIIn(int& Msg)
{
}

void _stdcall Wrapper::MsgIn(intptr_t Msg)
{
}

int _stdcall Wrapper::OutputVoice_ProcessEvent(TOutVoiceHandle Handle, int EventID, int EventValue, int Flags)
{
	return 0;
}

void _stdcall Wrapper::OutputVoice_Kill(TVoiceHandle Handle)
{
}
