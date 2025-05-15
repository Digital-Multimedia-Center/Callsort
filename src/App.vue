<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';

const inputPath = ref("");
const outputPath = ref("");
const columnName = ref("");
const message = ref("");

async function selectInputFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "CSV", extensions: ["csv"] }]
  });
  if (selected) inputPath.value = selected;
}

async function selectOutputFile() {
  const selected = await open({
    directory: false,
    multiple: false,
    filters: [{ name: "CSV", extensions: ["csv"] }]
  });
  if (selected) outputPath.value = selected;
}

async function sortFile() {
  if (!inputPath.value || !outputPath.value || !columnName.value) {
    message.value = "Please provide all fields.";
    return;
  }

  try {
    const result = await invoke("sort_csv", {
      args: {
        inputPath: inputPath.value,
        columnName: columnName.value,
        outputPath: outputPath.value
      }
    });
    message.value = result;
  } catch (err) {
    message.value = "Error: " + err;
  }
}
</script>

<template>
  <main class="container">
    <h1>LCC CSV Sorter</h1>

    <button @click="selectInputFile">Choose Input CSV</button>
    <p>{{ inputPath }}</p>

    <button @click="selectOutputFile">Choose Output Path</button>
    <p>{{ outputPath }}</p>

    <input v-model="columnName" placeholder="Column name (e.g. CallNumber)" />

    <button @click="sortFile">Sort CSV</button>

    <p>{{ message }}</p>
  </main>
</template>

<style scoped>
.container {
  max-width: 600px;
  margin: 5vh auto;
  text-align: center;
  font-family: sans-serif;
}
button {
  padding: 0.5em 1em;
  margin: 10px;
  border-radius: 5px;
  border: none;
  background: #396cd8;
  color: white;
  font-weight: bold;
}
input {
  padding: 0.5em;
  margin: 10px;
  width: 80%;
  border: 1px solid #ccc;
  border-radius: 5px;
}
</style>
