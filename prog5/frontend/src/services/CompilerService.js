import axios from "axios";

const API = axios.create({
  baseURL: "http://localhost:8080",
  withCredentials: false,
  headers: {
    Accept: "application/json",
    "Content-Type": "application/json",
  },
});

export default {
  doComprun(source) {
    return API.post("/comprun", { source: source }).then((response) => {
      console.log(response);
      return response.data.result;
    });
  },
};
