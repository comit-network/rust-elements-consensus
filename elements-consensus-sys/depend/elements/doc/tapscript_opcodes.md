This document proposes new opcodes to be added to the elements network along with the taproot upgrade. The new tapscript `OP_SUCCESS` opcodes allow introducing new opcodes more cleanly than through `OP_NOP`. In this document, we propose modifying the following `OP_SUCCESS`
to have the additional semantics. We use opcodes serially `OP_SUCCESS196`, `197`... in order
to avoid conflict with bitcoin potentially using `OP_SUCESSSx`(assuming bitcoin uses those serially based on availability). The initial version of this document had additional opcodes(`OP_FOR`, multi-byte opcodes) has since been updated to this current version in favor of application complexity.

# Resource Limits

## Changes in Taproot(Including Standardness Policy Changes)
Taproot already increases a lot of resource limitations from segwitv0, so there is no additional need to alter any of those. In particular, from BIP 342

- Script size limit: the maximum script size of 10000 bytes does not apply. Their size is only implicitly bounded by the block weight limit.
- Non-push opcodes limit: The maximum non-push opcodes limit of 201 per script does not apply.
- Sigops limit: The sigops in tapscripts do not count towards the block-wide limit of 80000 (weighted). Instead, there is a per-script sigops budget. The budget equals 50 + the total serialized size in bytes of the transaction input's witness (including the CompactSize prefix). Executing a signature opcode (`OP_CHECKSIG`, `OP_CHECKSIGVERIFY`, or `OP_CHECKSIGADD`) with a non-empty signature decrements the budget by 50. If that brings the budget below zero, the script fails immediately.
- Stack + altstack element count limit: The existing limit of 1000 elements in the stack and altstack together after every executed opcode remains. It is extended to also apply to the size of the initial stack.
- Stack element size limit: The existing limit of maximum 520 bytes per stack element remains, during the stack machine operations. There is an additional policy rule limiting the initial push size to `80` bytes.

## Additional resource limits changes in Elements

- New added opcodes `OP_MULTISCALAREXPVERIFY` for `k` base multi scalar exponentiation is counted as `50*k` units towards the SIGOPS budget. If the operation requires extra script_budget, the user must add additional witness elements to make sure that the script executes within the desired budget.

# New Opcodes for additional functionality:

1. **Streaming Opcodes for streaming hashes**: There is an existing limitation of `MAX_SCRIPT_ELEMENT_SIZE`(520 bytes) because of which we cannot operate hash functions like `OP_SHA256` on messages more than 520 bytes. This allows hashing on more than 520 bytes while still preserving the existing security against resource exhaustion attacks. The proposal for this is still under discussion in https://github.com/ElementsProject/elements/pull/817. We suggest the latest scheme suggested by Russell O'Connor
   -  Define `OP_SUCCESS196` as `OP_SHA256INITIALIZE` which pops a bytestring and push SHA256 context creating by adding the bytestring to the initial SHA256 context.
   - Define `OP_SUCCESS197` as `OP_SHA256UPDATE` which pops 1) a SHA256 context and 2) bytestring and push an updated context by adding the bytestring to the data stream being hashed.
   - Define `OP_SUCCESS198` as `OP_SHA256FINALIZE` to pop a SHA256 context and bytestring and push a SHA256 hash value after adding the bytestring and completing the padding.


2. **Transaction Introspection codes**: Transaction introspection is already possible in elements script by use of `OP_CHECKSIGFROMSTACKVERIFY`, however the current solutions are really expensive in applications like [covenants](https://github.com/sanket1729/covenants-demo). Therefore, we are not adding any new functionality by supporting introspection, only making it easier to use. The warning still remains the same as with covenants, if the user is inspecting data from parts of the transaction that are not signed, the script can cause unexpected behavior.
For opcodes that inspect data that is not committed in sighash, introspection is safe because any changes to the witness data would cause wtxid to change and it would revalidate the tx again.
   - Define `OP_SUCCESS199` as `OP_INSPECTINPUT` with the following semantics: Pop two minimal `CScriptNum`s from stack. The first number `n` denotes the type of introspection, and the second number `idx `denotes the position of input to inspect. Immediately abort if `idx` is out of bounds.
      1. If `n=0`, Push 1 byte "spend_type" onto the stack. spend_type (1) is equal to `(ext_flag * 2) + annex_present` as defined in [Modified BIP-341 SigMsg for Elements](https://gist.github.com/roconnor-blockstream/9f0753711153de2e254f7e54314f7169)
      2. If `n=1`, Push the outpoint_flag(1) as defined in [Modified BIP-341 SigMsg for Elements](https://gist.github.com/roconnor-blockstream/9f0753711153de2e254f7e54314f7169)
      3. if `n=2`, Push the outpoint as a tuple. First push the `txid`(32) of the `prev_out`, followed by a 4 byte push of `vout`
      4. If `n=3`, Push the `nAsset` onto the stack as two elements. The first push the assetID(32), followed by the prefix(1)
      5. If `n=4`, Push the `nValue` as a tuple, value(8, 32) followed by prefix,
      6. If `n=5`, Push the scriptPubkey(35) onto the stack.
      7. If `n=6`, Push the `nSequence`(4) as little-endian number.
      8. If `n=7`, Push the assetIssuance information(74-130) if the asset has issuance, otherwise push an empty vector
      9. If `n=8`, Push the annex onto the stack where the annex includes the prefix(0x50). If the annex does not exist, push an empty vector
      10. Otherwise treat as `OP_SUCCESS` and return true (without executing rest of script).
   - Define `OP_SUCCESS200` as `OP_INSPECTCURRENTINPUT` that pushes the current input index(4) as little-endian onto the stack
   - Define `OP_SUCCESS201` as `OP_INSPECTOUTPUT` with the following semantics:
      - Pop the stack pop as minimal `CScriptNum` as `n`. Next, pop another element as minimal `CScriptNum` input index `idx`.
      1. If `n=0`, Push the `nAsset` as a tuple, first push the assetID(32), followed by the prefix(1)
      2. If `n=1`, Push the `nValue` as a tuple, value(8, 32) followed by prefix
      3. If `n=2`, Push the `nNonce` as a tuple, nonce(32, 0) followed by prefix. Push empty vector for `None` nonce
      4. If `n=3`, Push the scriptPubkey(35).
      5. Otherwise treat as `OP_SUCCESS` and return true (without executing rest of script).
   - Define `OP_SUCCESS202` as `OP_INSPECTTX` with the following semantics:
      - Pop the stack pop as minimal `CScriptNum` as `n`.
      1. If `n=0`, Push the nVersion(4) as little-endian.
      2. If `n=1`, Push the nLockTime(4) as little-endian.
      3. If `n=2`, Push the number of inputs(4) as little-endian
      4. If `n=3`, Push the number of outputs(4) as little-endian
      5. If `n=4`, Push the transaction size in vbytes (4) as little-endian
      6. Otherwise treat as `OP_SUCCESS` and return true (without executing rest of script).

5. **Crypto**: In order to allow more complex operations on elements, we introduce the following new crypto-operators.
   - Define `OP_SUCCESS203` as `OP_ECMULSCALAREXPVERIFY`, pop the top element as `k`. Then pop next elements as points `G1`(first), `G2`(second)
      ..`Gk`. Next pop `k` scalars `x1`, `x2`... `xk`. Finally pop result element as as point `Q`. Assert `x1G1+x2G2+x3G3.. xkGk == Q` This counts as `k*50` towards budget sigops. If any of `G_i`,`Q` is point at infinity fail.
   - Define `OP_SUCCESS204` as `OP_TAPTWEAK` with the following semantics. Pop the first element as point `P`, second element as script blob `S`. Push the Taptweak on the top of stack `Q = P + H(P||S)*G`. If `|S| > MAX_ELEMENT_SIZE`, the user should use the streaming opcodes to compute the Hash function.

3. **Signed 64-bit arithmetic opcodes:** Current operations on `CScriptNum` as limited to 4 bytes and are difficult to compose because of minimality rules. having a fixed width little operations with 8 byte signed operations helps doing calculations on amounts which are encoded as 8 byte little endian.
   - When dealing with overflows, we explicitly return the success bit as a `CScriptNum` at the top of the stack and the result being the second element from the top. If the operation overflows, first the operands are pushed onto the stack followed by success bit. \[`a_second` `a_top`\] overflows, the stack state after the operation is \[`a_second` `a_top` `0`\] and if the operation does not overflow, the stack state is \[`res` `1`\].
   - This gives the user flexibility to deal if they script to have overflows using `OP_IF\OP_ELSE` or `OP_VERIFY` the success bit if they expect that operation would never fail.
When defining the opcodes which can fail, we only define the success path, and assume the overflow behavior as stated above.
   - Define `OP_SUCCESS205` as `OP_ADD64`: pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push a + b onto the stack. Push 1 `CScriptNum` if there is no overflow. Overflow behavior defined above.
   - Define `OP_SUCCESS206` as `OP_SUB64`: pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push a - b onto the stack. Push 1 `CScriptNum` if there is no overflow. Overflow behavior defined above.
   - Define `OP_SUCCESS207` as `OP_MUL64`: pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push `a*b` onto the stack. Push 1 `CScriptNum` if there is no overflow. Overflow behavior defined above.
   - Define `OP_SUCCESS208` as `OP_DIV64`: pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). First push remainder `a%b`(must be non-negative and less than |b|) onto the stack followed by quotient(`a//b`) onto the stack. Abort if `b=0`. Push 1 `CScriptNum` if there is no overflow. Overflow behavior defined above.
   - Define `OP_SUCCESS209` as `OP_LESSTHAN64`(cannot fail!): pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push ` a < b`.
   - Define `OP_SUCCESS210` as `OP_LESSTHANOREQUAL64`(cannot fail!): pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push ` a <= b`.
   - Define `OP_SUCCESS211` as `OP_GREATERTHAN64`(cannot fail!): pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push ` a > b`.
   - Define `OP_SUCCESS212` as `OP_GREATERTHANOREQUAL64`(cannot fail!): pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push ` a >= b`.
   - Define `OP_SUCCESS213` as `OP_EQUAL64`(cannot fail!): pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push ` a == b`.
   - Define `OP_SUCCESS214` as `OP_AND64`(cannot fail!): pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push ` a & b`.
   - Define `OP_SUCCESS215` as `OP_OR64`(cannot fail!): pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push ` a | b`.
   - Define `OP_SUCCESS216` as `OP_XOR64`(cannot fail!): pop the first number(8 byte LE) as `b` followed another pop for `a`(8 byte LE). Push ` a ^ b`.
   - Define `OP_SUCCESS217` as `OP_NOT64`(cannot fail!): pop the first number(8 byte LE) as `a`. Push `~a`.
   - Define `OP_SUCCESS218` as `OP_LSHIFT`: pop the first number as `CScriptNum` `l`(abort if l < 0 or l > 63) followed another pop for `a` (8 byte LE). Push `a << l` preserving the sign bit. `(-1 << 3) = - 8` returns fixed 64 bits, extra-bits are discarded and sign is preserved.
   - Define `OP_SUCCESS219` as `OP_RSHIFT`: pop the first number as `CScriptNum` `r`(abort if r < 0 or r > 63) followed another pop for `a` (8 byte LE). Push `a >> r`.(Sign bit is preserved).

4. **Conversion opcodes:** Methods for conversion from `CScriptNum` to `8-byte LE`, `4-byte LE`.
   - Define `OP_SUCCESS220` as `OP_SCIPTNUMTOLE64`: pop the stack as minimal `CSciptNum`, push 8 byte signed LE corresponding to that number.
   - Define `OP_SUCCESS221` as `OP_LE64TOSCIPTNUM`: pop the stack as a 8 byte signed LE. Convert to `CScriptNum` and push it, abort on fail.
   - Define `OP_SUCCESS222` as `OP_LE32TOLE64`: pop the stack as a 4 byte signed LE. Push the corresponding 8 byte LE number. Cannot fail, useful for conversion of version/sequence.

6. **Changes to existing Opcodes**:
   - Add `OP_CHECKSIGFROMSTACK` and `OP_CHECKSIGFROMSTACKVERIFY` to follow the semantics from bip340-342 when witness program is v1.


# General tips, suggestions and quirks for using Taproot opcodes

- In order to inspect the current input, `OP_INSPECTINPUT` can be used in combination with `OP_INSPECTCURRENTINPUT` to obtain information about input being spent on stack
- The input nNonce field is not consistently stored in elements UTXO database. Therefore, it is not covered in sighash or `wtxid` and hence introspecting it is not possible.
- Coversion opcodes can be used be used to convert ScriptNums/LE32 nums to LE64 for operations.