#include "../include/extern.h"
#include "./sub.h"
#include <stdio.h>

int main() {
  int a = 5;
  int b = 2;
  int res_add = add(a, b);
  int res_sub = sub(a, b);
  printf("%d %d", res_add, res_sub);
}
