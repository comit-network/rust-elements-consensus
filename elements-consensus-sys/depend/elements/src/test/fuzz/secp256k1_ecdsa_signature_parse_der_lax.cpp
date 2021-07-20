// Copyright (c) 2020 The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <key.h>
#include <secp256k1.h>
#include <test/fuzz/FuzzedDataProvider.h>
#include <test/fuzz/fuzz.h>
#include <test/fuzz/util.h>

#include <cstdint>
#include <vector>

bool SigHasLowR(const rustsecp256k1_v0_4_1_ecdsa_signature* sig);
int rustsecp256k1zkp_v0_4_1_ecdsa_signature_parse_der_lax(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_ecdsa_signature* sig, const unsigned char* input, size_t inputlen);

void test_one_input(const std::vector<uint8_t>& buffer)
{
    FuzzedDataProvider fuzzed_data_provider{buffer.data(), buffer.size()};
    const std::vector<uint8_t> signature_bytes = ConsumeRandomLengthByteVector(fuzzed_data_provider);
    if (signature_bytes.data() == nullptr) {
        return;
    }
    rustsecp256k1_v0_4_1_context* rustsecp256k1_v0_4_1_context_verify = rustsecp256k1_v0_4_1_context_create(SECP256K1_CONTEXT_VERIFY);
    rustsecp256k1_v0_4_1_ecdsa_signature sig_der_lax;
    const bool parsed_der_lax = rustsecp256k1zkp_v0_4_1_ecdsa_signature_parse_der_lax(rustsecp256k1_v0_4_1_context_verify, &sig_der_lax, signature_bytes.data(), signature_bytes.size()) == 1;
    if (parsed_der_lax) {
        ECC_Start();
        (void)SigHasLowR(&sig_der_lax);
        ECC_Stop();
    }
    rustsecp256k1_v0_4_1_context_destroy(rustsecp256k1_v0_4_1_context_verify);
}
