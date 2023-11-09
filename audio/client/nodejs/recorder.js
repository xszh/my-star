const ipmb = require('ipmb-js');

const { sender: ctrlSx, receiver: ctrlRx } = ipmb.join({
  identifier: "mystar.audio.adapter",
  label: ["control"],
  controllerAffinity: true,
  token: "",
});

const { receiver: dataRx } = ipmb.join({
  identifier: "mystar.audio.adapter",
  label: ["data"],
  controllerAffinity: true,
  token: "",
});

function control(cmd) {
  ctrlSx.send(
    {
      mode: ipmb.SelectorMode.Multicast,
      labelOp: new ipmb.LabelOp("control"),
      ttl: 0,
    },
    {
      format: 0,
      data: Buffer.from(new Uint8Array([cmd])),
    },
    []
  );
}

let recording = false;
let recordingTimer;

module.exports.start = () => {
  if (recording) {
    console.error('still recording');
    return;
  }
  recording = true;
  clearTimeout(recordingTimer);
  recordingTimer = setTimeout(() => {
    control(1);
    recording = false;
    console.error('1min limit, stop record and drop');
  }, 60 * 1000);
  control(0);
}

module.exports.stop = async () => {
  if (!recording) {
    console.error('not recording');
    return;
  }
  return new Promise((rs, rj) => {
    dataRx.recv(3000).then(({ bytesMessage }) => {
      const data = bytesMessage.data;
      rs(data);
    }, e => {
      console.error('recv error: ', e);
      recording = false;
      rj(new Error('record fail'));
    });
    clearTimeout(recordingTimer);
    control(1);
  });
}

module.exports.terminate = () => {
  ctrlRx.close();
  dataRx.close();
}