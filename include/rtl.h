// C ABI of rtl, the right-to-left example IME (bada-compatible v1.2; src/rtl_ime.rs, exported by
// export! in src/lib.rs). It lives in the same library as shout (libshout), under the prefix rtl_.
// Its Outputs and state carry "direction": "rtl" while the RTL row shows (docs §6.1).
#ifndef RTL_H
#define RTL_H

#include <stddef.h>
#include <stdint.h>

typedef struct RtlData RtlData;      // no data: pass NULL
typedef struct RtlEngine RtlEngine;

// Required.
RtlEngine *rtl_new(const RtlData *data);
void rtl_free(RtlEngine *engine);
const char *rtl_key(RtlEngine *engine, const char *event_json);
const char *rtl_set_mode(RtlEngine *engine, uint32_t mode);   // 0 muhenkan, else henkan
const char *rtl_flush(RtlEngine *engine);

// Optional, exported by rtl.
const char *rtl_catalog(RtlEngine *engine);
const char *rtl_state(RtlEngine *engine);  // read-only: what's staged, mode, menu, direction (§12)

#endif
