#ifndef WRENCH_DEAMON_CORE_H
#define WRENCH_DEAMON_CORE_H

#include "configuration.h"

extern AppConfig extract_arguments(argparse::ArgumentParser& program);

extern void run_app(AppConfig& config);

#endif  // WRENCH_DEAMON_CORE_H