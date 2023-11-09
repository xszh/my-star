const { start, stop, terminate } = require("./recorder");
const { recognize } = require("./asr");

const wait = (time) => new Promise((r) => setTimeout(r, time));

async function main() {
  try {
    console.log('start');
    start();
    await wait(3000);
    console.log('stop');
    const data = await stop();
    console.log('data', data.length);
    console.time('asr');
    await recognize(data);
    console.timeEnd('asr');
    console.log('result');
  } catch (error) {
    console.error(error);
  } finally {
    terminate();
  }
}

main();