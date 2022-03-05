<template>
  <CompilerComponent />
</template>

<script>
import CompilerComponent from "./components/CompilerComponent.vue";

export default {
  name: "App",
  components: {
    CompilerComponent,
  },
};
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
