#include "vm.h"


uint32_t trans_table[0x1000];

uint32_t page_map[0x8000];


void vm_seed_trans_table(void) {
    vm_seed(trans_table, trans_table, sizeof(trans_table));
}

void vm_seed_page_map(void) {
    vm_seed(page_map, page_map, sizeof(page_map));
}

void vm_seed(uint32_t *v_addr, uint32_t *p_addr, uint32_t length) {
    1;
}