const request = require("request");

module.exports.recognize = async (data) => {
  let requestUrl = "https://nls-gateway-cn-shanghai.aliyuncs.com/stream/v1/asr";
  requestUrl += "?appkey=" + "45KqS2ZhFlq6F9Sp";
  requestUrl += "&format=pcm";
  requestUrl += "&sample_rate=16000";
  requestUrl += "&enable_punctuation_prediction=" + "true";
  requestUrl += "&enable_inverse_text_normalization=" + "true";

  /**
   * 设置HTTPS请求头部
   */
  var httpHeaders = {
    "X-NLS-Token": "8c4b374597ad4d4598bdb8c9c847fcb0",
    "Content-type": "application/octet-stream",
    "Content-Length": data.length,
  };

  var options = {
    url: requestUrl,
    method: "POST",
    headers: httpHeaders,
    body: data,
  };

  return new Promise((resolve, reject) => {
    request(options, (error, response, body) => {
      if (error != null) {
        reject(error);
      } else {
        console.debug(body);
        if (response.statusCode == 200) {
          body = JSON.parse(body);
          if (body.status == 20000000) {
            resolve(body.result);
          } else {
            reject(new Error("asr fail: internal code: " + body.status));
          }
        } else {
          reject("asr fail - http code: " + response.statusCode);
        }
      }
    });
  });
};
