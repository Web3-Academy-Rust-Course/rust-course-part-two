import { ApiPromise, HttpProvider, Keyring } from "@polkadot/api";

const main = async () => {
    const provider = new HttpProvider("http://127.0.0.1:9944");
    const api = await new ApiPromise({ provider }).isReady;

    const keyring = new Keyring({ type: "sr25519" });
    const signer = keyring.addFromUri("//Alice");

    const amount = 100;
    const extrinsic = api.tx.defi.deposit(amount);
    await extrinsic.signAndSend(signer);

    console.log(`Alice has deposited ${amount} MUINT`);
};

main()
    .catch((e) => {
        console.error(e);
        process.exit(1);
    })
    .finally(() => {
        process.exit(0);
    });
