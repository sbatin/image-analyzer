<script>
  import Navbar from './Navbar.vue';
  export default {
    data() {
      return {
        path: '',
      }
    },

    methods: {
      navigate() {
        window.location.hash = `#images?path=${this.path}`;
      }
    },

    components: { Navbar },
  }
</script>
<template>
  <Navbar/>
  <div class="d-flex min-vh-100 justify-content-center align-items-center">
    <form class="row g-3">
      <div class="col-auto">
        <input type="text" name="path" class="form-control form-control-lg" placeholder="Path" v-model="path"/>
      </div>
      <div class="col-auto">
        <button type="button" class="btn btn-primary btn-lg mb-3" @click="navigate">Open</button>
      </div>
    </form>
  </div>
</template>