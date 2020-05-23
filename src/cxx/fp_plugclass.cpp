#ifdef _USE_AFX
#include "stdafx.h"
#endif
#include "fp_plugclass.h"

#if defined (__APPLE__) || defined (__linux__)
#include <string.h>
#endif

// destroy the object
void _stdcall TFruityPlug::DestroyObject()
{
    delete this;
}

TFruityPlug::TFruityPlug()
:	HostTag(0),
    Info(NULL),
    EditorHandle(0),
	MonoRender(false)
{
	memset(&Reserved, 0, sizeof(Reserved));
}

TFruityPlug::~TFruityPlug()
{
}
