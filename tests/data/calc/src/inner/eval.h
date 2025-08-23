#ifndef EVAL_H
#define EVAL_H

double eval_expression(const char *expr);
void set_variable(const char *name, double value);
void free_variables(void);

#endif
