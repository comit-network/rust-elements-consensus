
pub extern crate elements;

use elements::{confidential, encode::serialize, Script, Transaction};
use std::{error::Error, fmt};

/// The index exceeds our available inputs.
#[derive(Debug)]
pub struct IndexOutOfBounds;

/// The transaction does not correctly unlock the script.
#[derive(Debug)]
pub struct ConsensusViolation;

impl fmt::Display for IndexOutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "input index is out of bounds")
    }
}

impl Error for IndexOutOfBounds {}

impl fmt::Display for ConsensusViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "consensus rules violated")
    }
}

impl Error for ConsensusViolation {}

/// Verifies that the `coins` locked by `script` are unlocked by input `index` in `transaction` according to the consensus rules.
///
/// This function returns two layers of errors:
/// 1. The index may exceed the number of available inputs.
/// In some situations, it may be statically verifyable that the index is not out of bounds.
/// To make these situations more ergonomic, we provide it as a dedicated layer for safe use of `unwrap`/`expect`.
/// 2. The transaction may not correctly spend the script.
pub fn verify(
    script: Script,
    coins: &confidential::Value,
    index: usize,
    transaction: &Transaction,
) -> Result<Result<(), ConsensusViolation>, IndexOutOfBounds> {
    if transaction.input.len() <= index {
        // (1)
        return Err(IndexOutOfBounds);
    }

    let script = script.into_bytes();
    let coins = serialize(coins);
    let transaction = serialize(transaction); // (2)

    let mut err = 0;

    let ret = unsafe {
        elements_consensus_sys::bitcoinconsensus_verify_script_with_amount(
            std::ptr::null(),
            script.as_ptr(),
            script.len() as u32,
            coins.as_ptr(), // (3)
            coins.len() as u32,
            transaction.as_ptr(),
            transaction.len() as u32, // (4)
            index as u32,
            elements_consensus_sys::bitcoinconsensus_SCRIPT_FLAGS_VERIFY_ALL, // (5)
            &mut err,
        )
    };

    if ret == 1 {
        // (6)
        return Ok(Ok(()));
    }

    // if ret != 1, err is set
    match err {
        elements_consensus_sys::bitcoinconsensus_error_t_bitcoinconsensus_ERR_OK => {
            Ok(Err(ConsensusViolation))
        }
        elements_consensus_sys::bitcoinconsensus_error_t_bitcoinconsensus_ERR_TX_INDEX => {
            unreachable!("because of (1)")
        }
        elements_consensus_sys::bitcoinconsensus_error_t_bitcoinconsensus_ERR_TX_DESERIALIZE => {
            unreachable!("because of (2)")
        }
        elements_consensus_sys::bitcoinconsensus_error_t_bitcoinconsensus_ERR_AMOUNT_REQUIRED => {
            unreachable!("because of (3)")
        }
        elements_consensus_sys::bitcoinconsensus_error_t_bitcoinconsensus_ERR_TX_SIZE_MISMATCH => {
            unreachable!("because of (4)")
        }
        elements_consensus_sys::bitcoinconsensus_error_t_bitcoinconsensus_ERR_INVALID_FLAGS => {
            unreachable!("because of (5)")
        }
        e => panic!("unknown error code {}", e),
    }
}

#[cfg(test)]
mod tests {
    extern crate link_cplusplus;

    use super::*;
    use elements::encode::deserialize;
    use hex_literal::hex;

    #[test]
    fn index_out_of_bounds_protects_from_unreachable_1() {
        let transaction = Transaction {
            version: 2,
            lock_time: 0,
            input: Vec::new(),
            output: Vec::new(),
        };

        let result0 = verify(Script::new(), &confidential::Value::Null, 0, &transaction);
        let result1 = verify(Script::new(), &confidential::Value::Null, 1, &transaction);

        result0.expect_err("index equal to input list length");
        result1.expect_err("index greater than input list length");
    }

    #[test]
    fn mainnet_transaction_verifies() {
        let address = "H1YukBju4An78pumXgTcu31DyKwmYLqgi1"
            .parse::<elements::Address>()
            .unwrap();
        let transaction = deserialize(TX_HEX).unwrap();

        let result = verify(
            address.script_pubkey(),
            &confidential::Value::from_commitment(&hex!(
                "08b0faffc84b19acd0001e5c615d058d7bec15cce82253ac416300f52bbb4a2bf9"
            ))
            .unwrap(),
            0,
            &transaction,
        );

        result.unwrap().unwrap();
    }

    #[test]
    fn wrong_script_does_not_verify() {
        let address = "Gn1JvJW5KmfqZCUxaTo6ZgjP3yx7nDkP8v"
            .parse::<elements::Address>()
            .unwrap();
        let transaction = deserialize(TX_HEX).unwrap();

        let result = verify(
            address.script_pubkey(),
            &confidential::Value::from_commitment(&hex!(
                "091d24ec28f30a2fcdbefc46a8a3df21996e876de84a2dc0bb2068749334c54901"
            ))
            .unwrap(),
            0,
            &transaction,
        );

        result.unwrap().unwrap_err();
    }

    const TX_HEX: &[u8] = &hex!("02000000010249116dc31f764e98ac13cf4896aa78f232f5b481eae8bb8e73ccc0365c4db05b010000001716001453c93dfbc4a28163b2525a975691a37ec5b34f00feffffff7fe17612982f231d9bc7d3754df40f652ae6689c9f21e09dd5a7d0335e9c1c620100000017160014e47e86f9683df8793fb9ea52cd9a40e70b393383feffffff030b863f6bbdf4ec68f94b850f28d2ec0da3adf2eb3e74a6df1dcefdcf1d492277c6091d24ec28f30a2fcdbefc46a8a3df21996e876de84a2dc0bb2068749334c5490103d8bbde8c66ade0532d16ce7db8d4d07ab3628b5c69460ee6c283b5c7ab30edbf17a9145425c853ab527f2203b3426ed3765246d048d251870b9de74dbf092f1edf08b4c1b4674a69c25d9c63441f0e518d0a695efb7f37017409e63f4dbb43f8c52c67d06f8453373555e56b3f69026e48b792448a8dae065c7303f22d1c98ced11ff5dd019132987419557319785f6036728beb90b610c26e433517a9147813e0e37ff37ef3675192cdec79f43827dd52a287016d521c38ec1ea15734ae22b7c46064412829c0d0579f0a713d1c04ede979026f01000000000000010600002e8115000000024730440220727d299a1446435e0a6142931030794758d4adce4746dc2f4414345a8f8d5f4b02203d0e9bdafc535ee5f40531421ee688285b1c1becf771f28e097e2750e541ed280121022015072419d40375e4d7373be3e3404eddf809b7abfc041c04c57e5afa4044cc00000002473044022048b54d2dd0ebc7818047b45a44177d73aa1e4c596138c209096c623aa12d009202204d4ed70cf598f9e32f88c9631e80f41ffb30705a0c3d726c592fc8d016f5e3b2012103f51b380fc9319bfe6f320a7d83befc3d3c773e7d61a6b12dfd06d19d186498d8006302000328125d4d08462e21835100b80ee5ce501ecfd1ff9ae0816df827112d0e3d6e20798714b6aa2261ea70138297486b483e49a538ae07faa6fd3dfec425baddd1a42fe06883c9e0086bfadea31d6ebc78d767e92b579752849338f6b6ac45ad2414fd4e1060330000000000000001541e9e004a3a7a716956889f10024792d8e0be9fcae71f6e98c98b2d99bc45d8d40cd1594c296faa270bc615ce93df46b84de921b0142d75f9ebdbe55bc9f41dfb2b365c2cd313c40411d0c450692dda71da7802eac1f42837adbe6d999f0d9cef663f4d2f1615afa797ee2e28cb40920dad75e4d2e308da6aba7777aabb6e4d00b7de451f95ad36885fa59e68c4c8f7aa0d20dda9d5d792f26369c3ff8db7f1318a5f2f54701940b39d8d507ed213cf2fbc13f721db434808d5cb6bc228f6311e6da6260828ca4f009a97c6e5e7774b66eb39235b5a34de41643779e2c79f04803dfeda90e827e529fe762241a6395c9d105c475aa1ba13bd4a7a0f473df392223c9e5ae99094edfe043fd971133efd43ed0a0b861f4b8a007fc835a8516e47be3cbcd15aa72deda08e551a18ba1c5f036dbda96eb5526a65c40820ec6cab58d1447c17355468aac0ae6b7d6f8a2098472e03090acf834ae32f2095891bfa8068994a24085049e39ede036ae8dcb1fcefa3b60453a0efebf0e810134a422d992c1e3bf892db62d24d95b838718c00f13eff45aa0a6f2fc60a951ffda8b0b8ecca5c542ea6cdb4226bf1457dacbf34bde1c01baab7eb8275987a903738d0af56409741cc8c611be0d5a537a70088b1fa7c75fe0ab746fc98a2695e988c9bc4d0ecdd2d66eaf25c4aa9b710976e4bf4f6fc50e328201cf6bab1793c8dd6ccfb3b08b1a037ed1c2bfc8a4cbe2731a4f1615657c7645497c7237d482d79d33ae71001772ef4945d94ce2ded965c8b3eea63f73434a485c14e84b215d775a7c6553e74a97e7cc264568f7817595d4dd324ab6516fa492cd57c454f29fddb5178dfeb888ecd5297ad11f2ecc4f01898605a2f87732cb0a4cf687d8fc63d45ba98ae9c01d73b05953f7e8bb9a2982881c505f699337ba946a39982e691fa283c5f7f868d8495444a2277928af951ef7e874ca25474a72dfe78736d20148ead8a271c2088d2b91c59cea456fe3c592ef575083f7fa7f178b543fce85f689706bb0cbd037d4bef9775096676e6dab025326e94924e847ddb334b6acdfd519302c0a949c530b49c8c10f684b5807775a289d9bdfa33e0a52e5a4b9049e9577e4d3f07b6faf9854ae7be4a3080cf87615f55e1dca820c500d7d7fa509f07b47391592d2123fac991da7d502eb183cead434d0024fdaaa0e1dc4cb274ad557dd3677ba9672cd4f881f7388ad5d8d236101294b35c0cc42073296389a10b68219f1208eb27c747d0e0a390bfd846a4c9d4ca442068edbed8b5ebb968a2454efed785b6ee4c3c2cbfb6bd334c203ea7b2eff1419dffc4f432c1bb68e30927dcf831ab208af7e9aa62d404fd563ad2a4e3505f5c0f69d7f28896c6bce37e3fb55dcbd52f662629fc629395f0c542d50cedc59261a6f6116e37f6afd02bc637fc1a5975dc921039d8fd5458ed6b255d89f3459e4a698e5a163e6f1b12b0477891f0f7bdba66a15a673635356627d86d3d97fce3b2fc80ae44d98b0264af9423cc3fe9955fc62fb7912d145501fdf0840336992b44c7b2516971a7a9189f7de49f8c4c7975415235475f7045c3b1f82daa39f040082079895bfe68c350cd9a23c4cac6b8752caa7141c94e76f19471e61cbc94a92449d64e5cd0b67b35babd244109af6eda0f12f7ee9613dc2c869813d495ba813bd3fbd629d14eb7297fb655d01d5891ada3e2bd429c2dc58fd6fd149a77e01ec54d5bd5cc9de1fb0f164e9f2bc7208b9f8e9688260ea0b54faac4b5d4ddb5295c64b5b06f0634583cc672258d328c11b1250f069925c894624a7aaf768f3eb75af05098489add1cef477320dfb630cc5b449be8732c2b7a5c44f8b373cae6a1c4b192f56f018f37fb1e70f1e309f3a106f11407571521883f2cb59bdd099ac13dbfd709e24882c554060585913f14e2f064d1aa2b3f43a25d4f7f89d4518cdf3bdc68501d6b4a26cc4b05a2cce7c67a00358e8bd9e1587d3e26342d43ad146be5884bd4632525a46087cc6d3b430074e4ae7c29c844e03f98384ee3b147af12e79f20895b6500132f9cc55b20de48cfe49fef0045eb879621b0b36dcc1788fd29c1fa0385f5527bd42ec86a78d817109414ab909423e708e4bc917552e3cd87df965aca7b86c414a1eeeff7b1fbf2040303b29b3845dd0073ae8552ac0c52adb71f3f40264a679f536b45bdd5314613d4015d7b15aaffcb15c857887ce39a2584323bcb26d17f1b407a0af4b0899dc001c3f4a52632cfa84d903a6d22b59daf9f798037c5626be5fb49b3a1a2b7427b835dde1e04efea8197c5a9b5edb3060445945cc836e8e31f522e5f44d0d4ed8fdc85712c542c1dfc27159022ed2e0fb05e6751598a86f447dbf2320ff170f8a9e3ff0844c69f6c4972d152383a0ae7dd53ab51f37236b8e4a2f8e1aa3042d95bd5aac9cf57544dfc46162d0a59ccf3d121e5bba425cfa481ac08d2108621917e74de92cd2064153ef4810c89ab66f378c25cbcc43fadb1b9b088f2b2b1732ed23c2d58931d4aeafa158df708de7cb183416d801cd3a4f0fc64173663e671441b443ed76b6f280adfbdb239b95f43748b3b588e87341c860cf5ecaf3b2f1eddbe104e0472047a478a244c152db2e69a9239c9908a0f6c725ac363e25555c6fe85e622fcd72db86703ac4c7f106325276789dc9da2b19e5337d98e67a77a9ef9b8766ff314d9f519fc6788093e222bb8162cf455c675de92dd6698711ae3f1205c23c546d2dce7244c265d5aa33a19260f95410805618fc96e3903e7db03594c5f45542f7a818eb32a6770490bc88fdaecc42e6a6f01678af0c0103a753f1658afc2ed4696ea2e34c60199d8201f87c0af921fa46665ce0dde48c1947ab99d4e54766c3de2af18551c3a9b8f9f7e104304a740a785a89a2e35b1caf9c007dc884a2d370f262ff639b7dcb5f9b1ddb888ff6eacf9b861eaf61dbfe4995e988502c8f48e5668dbdd8784431ac5f39429ed6209652d765be459a6ac5a9e8890da4d111a676351c2364ac44534a39b20c1c74618fd7f63010253a61956331625038b0472954b345dd93ddce77eb9ec142689f5a9fda5deda7d0b62624ca51bcf81583977ba100745a57d1d9d0430037b925fcbb878daf741470bbd3f0fd206ea4a448f956272d939ed82a974d9bc3631ab7d4f9edde591758910e3104ae64699c5db308bff59333237fde663b570783f3e166ee9cb7d96fc803ae6b47869f57c4464d819cea62c9216afca5c09c4c62390e008819a77a7c8ab101e5560417c5a6ca83d1698e46bb3cdd8c91a9dac82b07deb5a0047908b41e6ab61f2c8c05f5a8c115374198b108bbf86439df89c4db4ab8e8d9b8ece2205c59eca33a69bc67397deae978fc4685ad82fc846c9c776d8ad67ac8efc94eeb5865bcc49fb00ed64758240f5b5bb4edd2614b69eb64ebbd9a49ee5dc8ccfb91ca836cdd9451ffba58b8a184420ef9e1695e2fc9d00c645743a41f04a4e9026832e701524ed3e9f9d37fc453a89be0f74f993c43d9fb13a3eda15477309660e9543130888bbba923be00130e7fdf91dc66ae1784643d5416055b506b290a2e2673d7e74b2993f6daa3e2cd2e1a8bbd4a9d6d3cd8d1d2a73831355289e8eee6ed7baff0bcf8701b5707b47423178ddb90c21e29adaf6061795de448209bd891ddefb480b10391b128b44bbb1443348552ba641eacd8723ab2eb8c6db7b16ba830e1402a946b5384e0bd9ab5087c16489ceb27cc6a07f55cd51b81cf4aa6416248dbbff85f5c566732b4745f745ca7cd72a97b48e2d6891ad3953dca4346528c0d13e090177dc42349168d4bc5f7ae57667151ec459e539522e2234698efb352a9ced12feead4bfeda7fc840eefda03e4a79625d4a2a1cedbddc4089a4b9c4dc4efec2ab3de51d531bbdf6bb0744429563c5dbce6bf4c606c6ed1b6aa658775174c2f77725ff78262b501661a529f877f00c17a719d4b7614795aab4d945867f2cf52990eb4f5e2087b79e162c73cdb7d7c7cb54bda1de5e12209a0a9080abddfe1cea83e18f0df2ad1facdb540395d174dc5a192878597a18ff00e6caaa6e404165eb91279ac4f452cfc7a0f22f04f61eeac474d15e5034ccf72fb3b8a989b8c2fa050d077250592d9613f30675f4a2a8ca701ba60ce3a831c25cff18cd9490f83793768770f699890470d033e80692682deb177e77c9b1b412c45c363544855b7d0118768d5ea5451c4c8eff3b5ddfdaf6fcc97c68254cc3227a1ddfa693d174df70d11f5f574588c147318b2975ad118839b7e0c24c1bc73077aec66141ba5628d66582d82a72757f4fb99e024b0fe21f76f0f27fa4940d241795bb4a551766c3a44e6805d18ed37a869a52ee7aba255e4b7ffdd2babeff24f55fcfff93ebc2af89378ea81e8f936b7ba4054aa38f802bb6603045433aec1666160ff77edb531a8cacab4085b4184920adc1bc9315d446eee36f1b80164fbec26d1cf1461e97861dfc677c3f5a5175f84d625cf1c970d36d6c1b884ec05a214dfcbb6d3956019e3aabcd7037b07a2836fcb3eee14dc39bd3c38bf3c3ca8050fc792295a5bf23efbf6677ef49bc03bbf59a984f6f1c6605515678a9a2758dc498385182004212e0977fdb813fe9e943ba2342800e65f0b7835ed9b8d4ffd2041af517d4a8d19868f5c44da54b86302018a5a7d0717ac8b0a91ba33aca0455ef375b199e86b94479fe345210822631840d08f97ec28df71020f738e0e734d5c12e5347e2a23cdaf2c593e4f1055c816350a851602f374a1f56fbdf5009954761abae365fc08420b96abfa6088455e83748821971be6f7e210990611816fe167c6b1b3ed1e2395f5e7492dc5e7491204390b33808f29d1afdce42da0aa264a6994d671c1fb24cd831f3d928f4d223936609c2483577c6006d062ff99c67ee52d569b81da0c121db8a4114aaf42159896a59da0174d33793e4c335fbe6ff1746d0debfeec8ac4667214218b429aa1cf6af28863c4672f70997a8b6ca296477e547bb36f6b1b15f5e2937aec7dbb0f57fc272af6446a1cc42bc4f2986bc780e3a8d99c211f9009cf038fb9cbb7b6a764552e746993a2c02cf4650a0abb802dbbfecaad3410d4ffcb963d80bb992f580fb9caed956f00e8efd6ea6614c56a758426f0e898d74ede665e047371a827951f0196cf8fa1970875816c78bbd362c1416ac9b307388c57a09f4712d197fe656ff483c40c3dcaa5e2835261b9e6c756f1a40684579680723ad922248195ab45e34f3dc718ac4e9d65aa181fddd28a0e3cba7b8ed8f38e5d6269395aead9b690b235f672b98d7ffd4a2d063b0ce3359203fb17fc64aea8d807af85982ac75566de50682011937854c278d78edb1ef327b5f58b1ecabacdd7b11f77bdd0d3c303773a2b62e4f6916212eebb1a6cf3b2f3338c53f66fc61891f3da4215176d238faef8236a58bfc111c9219ed164e3e8ead3b1adecd5409a09ca39cc14f8dc3c9a857d3aad1a7cda26b53b5fe4b1073baf9fb9f85b286f30cca4dcd871370a54f1206bd3491181f667ff6dae1b5e29e5485e2eaa0bcd63a5986b76ed845339398cd1a4c356a642fe6b42a3de8a5e4ae027958f6de197dfd05575b9f966b0584fc50b488ba93317093f54e30cb3e2f529476a5d20372c43eecfe1c91c69663d79872a7dea11845c9af55b573860c7fc65694ed03d061a5d2ad111ab89cb23e954c57600c8374f67a2828eafd146ef4dcd05719c40036f4470d3856b232fe23f5c437defa707049b38fd214dec5ba6661deefa2e360dd422193cb44e4cee221f8bd7bd7fc17c357570b4a7680c6a85f6dd0a2d35b471716a8dce93e24928a2a417b15aa776f7faa7ae0ff4dcf5c25d630200035cfd16548620b5cf486c494c8a0ff611f3da90250e254d52064786ee44e86a617d29a21715695787e6777440006340904f9917bbd617d684bd327dadf8df995ac63c8b6822fbe8f2ab51e2df49446c4b62f9014b08e851968be3aabe60e2bb34fd4e1060330000000000000001cb2e4b0176e1f141b9e66fffbbd8e2905d136bc646679bba4ec9cbce58537319a3a25068691d28a1a5aceb434f4b03f1cdeeb3ec2fa222ba4eac7d23c28af2b2cc8507771db22e5fcb43123a37cf1eb1b5dad998c7fd9605c8928702d82625ecf8c62a8718dfd1af529b0fde29aa4e180a08cb06ec773098aee78331b275c182682df611d66541fcca37b74310c9ecd14e9fba0c4be6245d36c7486885ba8da96c01b476def4e206f046d2f89dc21c7b2353e48e992668aa1ebc86ec60a72bcb0da919c53a4af7c5543a10cc497d8152e9c830c295f82b4b47cd44cb2f1f49692ae0e203e78ce62fb67f1aa9b6b131f287b169f6adacbfdb614fe6ee81fb993c3a942000f7ecc1f42a00c00e1c6706c8831cee544e9b660b437ea5d4a66d7590e471231d54e897e5ea457094d160457dbe9b3e54d5c693635106c9852df6c4b4c18dd5ee21947f94ae7060c6951477b327dad60ceaead3c4448ce204a8228e07d2d4d6051c33438bb527551690f65e3a502bd3888826026358ee199c9409b7a8b19abdbefb1c1aca653039812307e049dd5b69dc1ed0ffc7651a82d12f35f464a6ffe805f33fa288470404556b23a10a70299520ae629287ad52d24c6f53af01fe4695b2b2fe268ac855b53863095749fc3b5e96a68dfd5a95c3b8dc6ca50e245cb85c66a0b5e0a9e1202382b4ddc14d01f3ccda7c029b641e98ce138bd7292245a002dba397b183e95c44ea622ec535db837353278232aefa56f9e562151287cef2e9309675699b96b3e08b0c8e55e14c737fdb7d3a36725674756a4526fdc18af0ecbb36f53e41575370c797ff4653ffaf9bb9afcdbc78b6c9ef9fe41698f65249375c10706e13f7ce6d023172f58eb5947861e9a13864e496d4bf92cc81e1c7408d6673978f9eedecedf1ab8ea09e123878c66552e28a35e799ff3c6d7b6dcf037d403991cabd388ae68b3d9422bbb22538cfe7ebaf80f21a6367a5cc8a4439f48da6bf2403b13567d7e747973c249505f6de496725ef340e8f7664313fc9404e440e63b5e0169d680df384c1f22c6f94081573e1d15520cf56a562e42d83c4c335c8a06d94a6d97aa9a6baea9531d998741b606c7328f3c5e4825fa0bb276a1b4881e4274ce14038fb020480e7726112b89a44f77fec2e55913bd6805e0b45da5372f3c238aa1f050f7e880a2ee77a39ccdf4d278bb8664e90bbfe2e8251e0b3060a97855753e043894d54bc8cb7e75a1a6be5821c24ea4861bf48da77a6a16afe3258a3b522dc6f72a91feed15f42089bf7a90e285de96f3be76e5c8428b5f233626670e6445021b1cab40c006a41367f9547656492eead5d0180ff488195ab3d52cac14ad1a1a84aef58510ab3cdeaa07a856022022d2ccb7feb1a3f1453962e4121ba3ae5ad1e47599a2351260b37fba700ab01c18a374b2d8fc6e11df09d5d860f640606ecbc14d24d23493b66bb021ace21073f2aad9125d50090ab6ea9fa1811e39e2b5924a09e9ca689749ee8409c7e7e2f856dba20917445b47de79d2d3c88a064e0a43ab5fd00444fe35aea6c264738e38c29b5e6a0026d64c668d51cd2bb7bffdc3413309068baf8f9cda16cdb3d97695ce865ffbb23824628abd1bad1a5999bd40b0662453111714aa0cb464bc86fa47065044dcbe3d825be058419a7eee457a4b817d5af2e070d3802a90db38dadf3dcdd7179d576fb7da8a990927d0b05dc8efd4bdaf4814ca9648e11140979f31e9181e0c6f4bee7490ad3545c486871b84f7300daee052eb5b2d3fae81ed3cf73d409759b350727e5c3633deb044f53fca9aaa184ec3dc7fb8603179ee91e10615d39e2d09cd132209bb65b3ef15806ff888a69520e033337ab0039f8bad686f6c134b7b7bdc1e56c4aa7abd1c9aae7979358353302effc5c8330952430972a1ebe4e858ba33d6bb9081fa3b7030344110e5028b097ce93a3b418d5855be631c152750b1703f7d37dc6f38887d603b573c9b512b5bab8df46367585026930fe5a0f5a0b8cd02d610f89f4b60caa695620adb155d4c8b1645ce5d1d32404bcdad469fade5e67e56974a483e3ed02e5633e2d03dc95001f08c194bfcb08661dfc86f19e91b0d428350dbf31ead3202171d788f7524928c1f915d98b773e6fbfad542830d001f1d6ec020b261a1538932852958352a4ac9a4a0cdd0876362c10e1f30a3d14925659a7a1ed946d8f350f884866b6fa27aaac7f58e4c90c23fceb442ad8d1e3b49b9752bcbfc17a9737944120d6d225ab4c37e61ba84382632fc72d0a5ccf5d88c07a951b7f57bc4d3869077cf3b28c562d623f39f0a23ff1dc94a2646b293a418d2d51e963ac21be04ba1f7ffe3fb7fda9cdcdcc72a7ee6b4d46bcd5174378db90c30744a69eb345075571b8fa3800fbb36d56f258fc589f13fd2816604a0aabb5069839cfb8805153d71361ed53561a6a9f135bd23777d3e5596434522700d31d9d46b6e202148ff498493655110bc64bcff56ccf38a1fa3a1a0f066f0fd1bb801a68a09b6e36d83e4abb38d2c0f6b4671ba366141febd436882da9aad5a9f292d909e2dac5fba8f457b46c6e49be9565e5f74bda9a89f53879fb0f03e6c9033b71d7dc5dd3931a28f7f70fc092164d06b9c821c87807e71eb04bd3e55043b2bd54f9c81d203ab19b8814a904cf643ef0e233bcf93a17bb089bb809a992cef55066337ea476d1ca91ffc8e87419f7af01db0f7cd8a284c5ca9fbb9d445287bc8baf253cad1ee01dc3f13be69e667617ef17c7f3e6620e8652110c8071e93222e50501399f41910951dfa133cbe024cde44c9cbdc07c76d646cf7700775aa9589ec84e7b67441effb6317aa03dc54ce7a86703ae589065b0f7736a9bac2f2b70adfdd79fe4baedef4f0da3313373d9883ebf6b40168be02d492f01b70f7250270668aa83a93a701a39706f4e32083221935a30d3898fd37c992878e6e23367c98d4685fea529258bb5eef0ad556aeda5ea591547228667acc5dfa8f4a80ebf4deefb6e7b0986c5f95c845cf30e8d7c2ed4c242d29b9d981d2983413a421df6a8fd0f6e067eb111ac742c432ea0f75c8716fd702620de02a03647e34a91b0839d1c2bcefb5c162e080abd622fb51567c537672350087d69683919e28ebde32ced8b1daaf16ff8edd09238d9764d154b2602bd768836a4e4b5a92d0c9cee8badf91f5a84f5717f83b615f8a9f23061ed87b72e27549cd1567d1d6c75c5fba43e7191c03c7bb9668341ff6dc10e548c3ab06a869c70dfcad08418cb1faafc6f53b9c264b544d7082ed22841e37a8776ccc2793c94d553b14d9402b56aee7b6d9a0cb0ec8ba4f74f9f8f768a89a1455a24615b5db419e65f6e1d93ecb798d7fa07403573594bc48239fbff67ebfbe135cacec06570aa347c57d81f6bd9ba6b37ce014d1c7e0e67ff07aa50d4d18a69715d050ddb38c804710522926c8e4f45385ec5c8268d25035938f109a3fd8aaa0766331e1d7f89d7124ee504ad8c77db4b7f5d910b2641dc3165b88fe10921946c36df59b25b60d9884375be55a7076eb93c309ca448d41cd5d03ab20a228f89a83ee5c7eecb7918c55db59150e7b0c9a9f0b05ed44b35b7f576d8570b64af2bef1db40e0fbbffa735e978535531f11df09900ebd178a6386eddce7db272494c8a63df82a857966bd28ec7c6c3c797b61b031a9378708ec19a0cc3ff8edea5ae7505d9e844d96bfee8f448e17629afabfec2c45faf61f00609f07162733221f01c35b71204c2f748c588f3547188eed73be2bc24af77ebe153a7e3a79a20a05bc3261022b9a0f181b4fa8c3282c9b4d544edf86c4ee04b74d68fbed53ecf282e2557669edd1fba6a3cff338064a8627713468ed8f9a8c3438fdff27ff2e65788f0c5675f3a99cdd778c29f3dca6745404ec18f0a714f1f80addcb28f0e4cdfcfa2a552ba954aad29deb69b3e4a3d48b7ca3ea58266adeb33411b20de5267caa99f1558b9200c84f0e208ce5f64eee0568f8099284db8d348ee8abab5bc5a9b351d9b2bef17308d2d4e39b368a4f7ce57b24cd0b7818729785616261ec45d1ff4278aa04b030949200200f62d4f07a095415b62be26f6a658d63a64aa41c3e057b83368ee7be9afbe05b697ab97e7b99de6b02528d208836dd8c9170a2873da38194f46645496c22d61ce3a8d603ce5b097ef1431da3dd5374fa092a2e35b0ae2fb30c13cb749a74771e847a7aca20073264d4d32dfa70839587126cd84b34a288ad5a9be9ebdcc531f143c626db2e4efb2e7e6fffe0b14ff30e544fa2724df1b4cef3f34f874436189df9757878f1a1ba7894c0bc0a62e1032de261313fe2358204f87e66104f228233637c4ab4a462a90a13994211298227037149d8870ea4591cb26e96ae2339ad21afa1f1226d00319989483b60d0306d72f96b516ec9eab46ddba43add5c77a6aded14179d609a60ed27cf3c026b8b1ac0f030ad4784f95b5b6ba6aeed2437a3709d8a8a6991c128eebe31198c882269cb6905296c0bdb0c1524287fac0d67b19648e9fbd952e6a3959112a01b553b48600def126f0752f44f82d78f8234234ba69c003eadc078ec81ec0d8908700f5efa16260a3ca8c5c9fb6f5fe9a00b3d45dc5aab07e9d40651f4768ccf65060f57e78f689f59172181806034232d3c1774eb0fe94ccda31453921525a2b914aabf57614253070e6fa63ebf89a19c0075089bdbdf7cac33923683f63ae4e94c26114c3198e43fa3e7b1b96623842271f9b2cef9b6cccce184e810da3b776d488dfc25a4cd4822a6fc29b8933a6e8e4f635818de7945d303b8ecdadb2f0e887d7a75d6bbaba89d0f316a093c5e9ab88d7ddc45e0cc17fd28b9caca2f41f0937ab5b9520a42325c024ef2ac44a93ca616ebe9c619581a8741c1f23d2bcdd2343bbb25658b819ba4d1fbef1e1fa4c8f5d108ae5a72f6fb428e4e742bc33b41665721f99c7195e3281153486b5502d43dc875d226e36239def5dae1278fd792a2330ec251105d3bb75baf8a329da46ab93c62654e5a17256e243374b1a7d7b4edbd65dd6bdcb3aa531c7410cab053efaffed62833353900893d29dfaec005b898941896d885c19c537ea7c6fb4a1e49a07c8ee765ffe580ffe0c3ba85777a1ef93104e1083d8e2e151d5cfc17a88600a4381b3a0ac8c40c3398930bcef28adf74aaeee26b830d4c2d2d86e6ba9156b59ea59279a13ac1de3d4e5468741243b235f3ebc6cb939f204458c9ab2dc509db52b68fc73a5ca8ec8d202248d613d15f6aa5d874175a68f1ce8e4928c17dafa52ab77e6ee10468ec59703b574e63cf34831b094699e25e8eaf834acec57410d9ac49216f573f503dae83c11c1671edbaa878bbcb7bee970423f20d7e0bd2b6983f360d97a9b34aa55a4d22cf6e74a97ca3818324e02c516cb81f4033b8f6f71bff17052228996dd099cce5c784c26ceaf1558c53a6bdc1a4c47cff91254a111e4186b56a3490d77de75ccb38c3456e67bacf58928fa8ca1fc4fad354dbcb132971e89ab1a43c69036aed881163ba87cd40042775d1d5fbc3998cf9b146111e1cef0b8b0f003bc28245b5f06365d8486e58980731daf215c94ece8ad4b53b58ba31a47be9f2d7f43fd7dfa2f306cada33f245646e5c5bf81e4668ebf63ac0cb9d23c142e7e0d7b2d132a93b6416a5472a5c95cb448d4b1a9a3c8810847a3a12d3a78dca1daaa826d921f78b82939c34423ca35769feeab5e66d528bd89af3d0e390c259e3f7671ffaf9a3d34f9500c91464f06004d0ac701a89d137a94834267b0860ee81e45d41e19e878940a5f95aad3b8b953d54cfb56cc158f79d7c230bde3aa120030f92369e47a1ec0b3ebfa0b574a9076e7a9c5400000");
}
