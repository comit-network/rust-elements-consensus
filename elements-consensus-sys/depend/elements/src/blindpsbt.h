// Copyright (c) 2020 The Elements Core developers
// // Distributed under the MIT software license, see the accompanying
// // file COPYING or http://www.opensource.org/licenses/mit-license.php.

#ifndef BITCOIN_BLINDPSBT_H
#define BITCOIN_BLINDPSBT_H

#include <blind.h>
#include <key.h>
#include <pubkey.h>
#include <primitives/transaction.h>
#include <primitives/confidential.h>

#include <secp256k1.h>
#include <secp256k1_rangeproof.h>
#include <secp256k1_surjectionproof.h>

struct PartiallySignedTransaction;

enum class BlindingStatus
{
    OK, //!< No error
    NEEDS_UTXOS,
    INVALID_ASSET,
    INVALID_ASSET_COMMITMENT,
    SCALAR_UNABLE,
    INVALID_BLINDER,
    ASP_UNABLE,
};

std::string GetBlindingStatusError(const BlindingStatus& status);

bool CreateAssetSurjectionProof(std::vector<unsigned char>& output_proof, const std::vector<rustsecp256k1_v0_4_1_fixed_asset_tag>& fixed_input_tags, const std::vector<rustsecp256k1_v0_4_1_generator>& ephemeral_input_tags, const std::vector<uint256>& input_asset_blinders, const uint256& output_asset_blinder, const rustsecp256k1_v0_4_1_generator& output_asset_tag, const CAsset& asset);
uint256 GenerateRangeproofECDHKey(CPubKey& ephemeral_pubkey, const CPubKey blinding_pubkey);
bool CreateValueRangeProof(std::vector<unsigned char>& rangeproof, const uint256& value_blinder, const uint256& nonce, const CAmount amount, const CScript& scriptPubKey, const rustsecp256k1_v0_4_1_pedersen_commitment& value_commit, const rustsecp256k1_v0_4_1_generator& gen, const CAsset& asset, const uint256& asset_blinder);
void CreateAssetCommitment(CConfidentialAsset& conf_asset, rustsecp256k1_v0_4_1_generator& asset_gen, const CAsset& asset, const uint256& asset_blinder);
void CreateValueCommitment(CConfidentialValue& conf_value, rustsecp256k1_v0_4_1_pedersen_commitment& value_commit, const uint256& value_blinder, const rustsecp256k1_v0_4_1_generator& asset_gen, const CAmount amount);
BlindingStatus BlindPSBT(PartiallySignedTransaction& psbt, std::map<uint32_t, std::tuple<CAmount, CAsset, uint256, uint256>> our_input_data, std::map<uint32_t, std::pair<CKey, CKey>> our_issuances_to_blind);

#endif //BITCOIN_BLINDPSBT_H
