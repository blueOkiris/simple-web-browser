#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// Convert AdGuard filter rules to WebKit JSON format
char *adguard2webkit__convert(const char *rules_ptr);

#ifdef __cplusplus
}
#endif
