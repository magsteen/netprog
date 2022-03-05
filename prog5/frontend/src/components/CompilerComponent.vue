<template>
  <div>
    <textarea name="" id="" cols="30" rows="10" v-model="source"></textarea>
    <button @click="submit">Submit</button>
    <div id="outputBox">
      <p id="output">{{ output }}</p>
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

<style scoped></style>
