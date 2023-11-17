const axios = require('axios');

axios.get('http://alicloute-token-rpc-shanghai-tnhzbmofml.cn-shanghai.fcapp.run').then(d => console.log(d.data), console.error);