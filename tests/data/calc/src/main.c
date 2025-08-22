#include "eval.h"
#include "util.h"
#include <stdio.h>
#include <string.h>

#define INPUT_SIZE 256

int main(void) {
  char input[INPUT_SIZE];

  printf("MiniCalc v0.1\nType 'quit' to exit.\n");

  while (1) {
    printf("> ");
    if (!fgets(input, INPUT_SIZE, stdin)) {
      break;
    }

    trim_newline(input);

    if (strcmp(input, "quit") == 0) {
      break;
    } else if (strncmp(input, "let ", 4) == 0) {
      char var[32];
      double val;
      if (sscanf(input + 4, "%31s = %lf", var, &val) == 2) {
        set_variable(var, val);
        printf("Set %s = %g\n", var, val);
      } else {
        printf("Invalid assignment. Use: let x = 42\n");
      }
    } else {
      double result = eval_expression(input);
      printf("= %g\n", result);
    }
  }

  free_variables();
  return 0;
}
