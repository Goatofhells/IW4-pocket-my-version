#include <stdint.h>
char *iw4_verify(const char *root, const char *catalog);
char *iw4_extract(const char *archive, const char *destination);
uint32_t iw4_cache_limit(uint32_t requested, uint64_t physical_mb);
void iw4_string_free(char *pointer);
