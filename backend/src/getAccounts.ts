import { ApiPromise, HttpProvider } from "@polkadot/api";

const main = async () => {
    const provider = new HttpProvider("http://127.0.0.1:9944");
    const api = await new ApiPromise({ provider }).isReady;

    const users = await api.query.defi.accounts.entries();

    if (users.length === 0) {
        console.log(users);
    }

    for (let [keys, data] of users) {
        let account = keys.toHuman()?.toString();
        let readableData = data.toHuman();

        console.log(account);
        console.log(readableData);
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
