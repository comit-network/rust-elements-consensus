// Copyright (c) 2011-2019 The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#ifndef BITCOIN_WALLET_COINCONTROL_H
#define BITCOIN_WALLET_COINCONTROL_H

#include <asset.h>
#include <chainparams.h>
#include <optional.h>
#include <outputtype.h>
#include <policy/feerate.h>
#include <policy/fees.h>
#include <primitives/bitcoin/transaction.h>
#include <primitives/transaction.h>
#include <script/standard.h>

const int DEFAULT_MIN_DEPTH = 0;
const int DEFAULT_MAX_DEPTH = 9999999;

//! Default for -avoidpartialspends
static constexpr bool DEFAULT_AVOIDPARTIALSPENDS = false;

/** Coin Control Features. */
class CCoinControl
{
public:
    //! Custom change destination, if not set an address is generated
    std::map<CAsset, CTxDestination> destChange;
    //! Override the default change type if set, ignored if destChange is set
    Optional<OutputType> m_change_type;
    //! If false, only selected inputs are used
    bool m_add_inputs;
    //! If false, allows unselected inputs, but requires all selected inputs be used
    bool fAllowOtherInputs;
    //! Includes watch only addresses which are solvable
    bool fAllowWatchOnly;
    //! Override automatic min/max checks on fee, m_feerate must be set if true
    bool fOverrideFeeRate;
    //! Override the wallet's m_pay_tx_fee if set
    Optional<CFeeRate> m_feerate;
    //! Override the default confirmation target if set
    Optional<unsigned int> m_confirm_target;
    //! Override the wallet's m_signal_rbf if set
    Optional<bool> m_signal_bip125_rbf;
    //! Avoid partial use of funds sent to a given address
    bool m_avoid_partial_spends;
    //! Forbids inclusion of dirty (previously used) addresses
    bool m_avoid_address_reuse;
    //! Fee estimation mode to control arguments to estimateSmartFee
    FeeEstimateMode m_fee_mode;
    //! SigningProvider that has pubkeys and scripts to do spend size estimation for external inputs
    FlatSigningProvider m_external_provider;
    //! Minimum chain depth value for coin availability
    int m_min_depth = DEFAULT_MIN_DEPTH;
    //! Maximum chain depth value for coin availability
    int m_max_depth = DEFAULT_MAX_DEPTH;

    CCoinControl()
    {
        SetNull();
    }

    void SetNull();

    bool HasSelected() const
    {
        return (setSelected.size() > 0);
    }

    bool IsSelected(const COutPoint& output) const
    {
        return (setSelected.count(output) > 0);
    }

    bool IsExternalSelected(const COutPoint& output) const
    {
        return (m_external_txouts.count(output) > 0);
    }

    bool GetExternalOutput(const COutPoint& outpoint, CTxOut& txout) const
    {
        const auto ext_it = m_external_txouts.find(outpoint);
        if (ext_it == m_external_txouts.end()) {
            return false;
        }
        txout = ext_it->second;
        return true;
    }

    void Select(const COutPoint& output)
    {
        setSelected.insert(output);
    }

    void SelectExternal(const COutPoint& outpoint, const CTxOut& txout)
    {
        setSelected.insert(outpoint);
        m_external_txouts.emplace(outpoint, txout);
    }

    void Select(const COutPoint& outpoint, const Sidechain::Bitcoin::CTxOut& txout_in)
    {
        setSelected.insert(outpoint);
        CTxOut txout;
        txout.scriptPubKey = txout_in.scriptPubKey;
        txout.nValue.SetToAmount(txout_in.nValue);
        txout.nAsset.SetToAsset(Params().GetConsensus().pegged_asset);
        m_external_txouts.emplace(outpoint, txout);
    }

    void UnSelect(const COutPoint& output)
    {
        setSelected.erase(output);
    }

    void UnSelectAll()
    {
        setSelected.clear();
    }

    void ListSelected(std::vector<COutPoint>& vOutpoints) const
    {
        vOutpoints.assign(setSelected.begin(), setSelected.end());
    }

private:
    std::set<COutPoint> setSelected;
    std::map<COutPoint, CTxOut> m_external_txouts;
};

#endif // BITCOIN_WALLET_COINCONTROL_H
