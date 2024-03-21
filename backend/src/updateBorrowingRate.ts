import * as dotenv from "dotenv";

import { ApiPromise, HttpProvider, Keyring } from "@polkadot/api";
import { checkExtrinsicStatus, ResultDto } from "./util";

dotenv.config();

const main = async () => {
    const provider = new HttpProvider("http://127.0.0.1:9944");
    const api = await new ApiPromise({ provider }).isReady;

    const keyring = new Keyring({ type: "sr25519" });
    const signer = keyring.addFromUri("//Alice");
    // const signer = keyring.addFromMnemonic(
    //     process.env.PRIVATE_ACCOUNT_MENMONIC!
    // );

    const newRate = 1;
    const extrinsic = await api.tx.defi.updateBorrowingRate(newRate);

    await extrinsic.signAndSend(signer);

    const result: ResultDto = await checkExtrinsicStatus(
        api,
        "defi",
        "updateBorrowingRate",
        signer.address
    );

    if (result.status) {
        console.log(`Borrowing rate has been changed to: ${newRate}`);
    } else {
        console.log(
            `An error occured while trying to change the borrowing rate. Error: ${result.message}`
        );
    }
};

main()
    .catch((e) => {
        console.error(e);
        process.exit(1);
    })
    .finally(() => {
        process.exit(0);
    });
