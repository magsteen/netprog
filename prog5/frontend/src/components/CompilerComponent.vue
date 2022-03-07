<template>
  <div id="compiler">
    <textarea
      name="input"
      id="inputbox"
      cols="30"
      rows="10"
      v-model="source"
    ></textarea>
    <button @click="submit" id="submitButton">Submit</button>
    <div id="outputbox">
      <div class="output">Run result:</div>
      <div class="output">{{ output }}</div>
    </div>
  </div>
</template>

<script>
import CompilerService from "@/services/CompilerService.js";

export default {
  name: "CompilerComponent",
  data() {
    return {
      source: "",
      output: "",
    };
  },
  methods: {
    async submit() {
      this.output = await CompilerService.doComprun(this.source);
    },
  },
};
</script>

<style scoped>
#compiler {
  width: 1200px;
  height: 600px;
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: center;
}

#inputbox {
  width: 400px;
  height: 400px;
  margin: 10px;
}

#submitButton {
  border: 0px;
  margin: 10px;
  background: blue;
  width: 100px;
  height: 50px;
  font-weight: bold;
  font-size: 150%;
  color: white;
}

#outputbox {
  width: 400px;
  height: 400px;
  margin: 10px;
  background: lightcyan;
  border-radius: 10px;
}

.output {
  overflow: auto;
  align-content: flex-start;
  text-align: left;
  padding: 10px;
}
</style>
