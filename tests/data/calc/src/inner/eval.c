#include "eval.h"
#include "../util.h"
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct Var {
  char *name;
  double value;
  struct Var *next;
} Var;

static Var *vars = NULL;

// Simple linked-list variable table
void set_variable(const char *name, double value) {
  Var *v = vars;
  while (v) {
    if (strcmp(v->name, name) == 0) {
      v->value = value;
      return;
    }
    v = v->next;
  }
  v = malloc(sizeof(Var));
  v->name = strdup(name);
  v->value = value;
  v->next = vars;
  vars = v;
}

double get_variable(const char *name) {
  Var *v = vars;
  while (v) {
    if (strcmp(v->name, name) == 0) {
      return v->value;
    }
    v = v->next;
  }
  return 0.0;
}

void free_variables(void) {
  Var *v = vars;
  while (v) {
    Var *next = v->next;
    free(v->name);
    free(v);
    v = next;
  }
  vars = NULL;
}

// Very minimal parser: supports + - * / and variables
double eval_expression(const char *expr) {
  double acc = 0;
  double num = 0;
  char var[32];
  char op = '+';
  const char *p = expr;

  while (*p) {
    if (isspace((unsigned char)*p)) {
      p++;
      continue;
    }

    if (isalpha((unsigned char)*p)) {
      sscanf(p, "%31[a-zA-Z]", var);
      num = get_variable(var);
      p += strlen(var);
    } else if (sscanf(p, "%lf", &num) == 1) {
      while (*p && (isdigit((unsigned char)*p) || *p == '.'))
        p++;
    } else {
      p++;
      continue;
    }

    switch (op) {
    case '+':
      acc += num;
      break;
    case '-':
      acc -= num;
      break;
    case '*':
      acc *= num;
      break;
    case '/':
      if (num != 0)
        acc /= num;
      break;
    }

    while (*p && isspace((unsigned char)*p))
      p++;
    if (*p && strchr("+-*/", *p)) {
      op = *p;
      p++;
    }
  }

  return acc;
}
