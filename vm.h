
#include <stdint.h>

void vm_seed_trans_table(void);
void vm_seed_page_map(void);
void vm_seed(uint32_t *v_addr, uint32_t *p_addr, uint32_t length);

