/*
protoc --cpp_out=. thing.proto
clang++ -fsanitize=fuzzer -I/usr/include/libprotobuf-mutator thing.cc thing.pb.cc -lprotobuf -lprotobuf-mutator-libfuzzer -lprotobuf-mutator
*/

#include <cmath>
#include <iostream>

#include "thing.pb.h"
// libprotobuf-mutator
#include "port/protobuf.h"
#include "src/libfuzzer/libfuzzer_macro.h"

protobuf_mutator::protobuf::LogSilencer log_silincer;

DEFINE_PROTO_FUZZER(const thing::Msg &message)
{
    // Emulate a bug.
    if (message.optional_string() == "abcdefghijklmnopqrstuvwxyz" &&
        !std::isnan(message.optional_float()) &&
        std::fabs(message.optional_float()) > 1000)
    {
        std::cerr << message.DebugString() << "\n";
        abort();
    }
}