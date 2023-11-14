var RPCClient = require("@alicloud/pop-core").RPCClient;
var fs = require('fs');
const [ak_id, ak_secret] = fs.readFileSync('./AccessKey.csv', 'utf-8').split('\n')[1].split(',');

var client = new RPCClient({
  accessKeyId: ak_id,
  accessKeySecret: ak_secret,
  endpoint: "http://nls-meta.cn-shanghai.aliyuncs.com",
  apiVersion: "2019-02-28",
});

// => returns Promise
// => request(Action, params, options)
client.request("CreateToken").then((result) => {
  console.log(result.Token);
  console.log("token = " + result.Token.Id);
  console.log("expireTime = " + new Date(result.Token.ExpireTime * 1000));
});
