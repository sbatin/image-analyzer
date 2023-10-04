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
      },
    },

    components: { Navbar },
  }
</script>
<template>
  <Navbar>
    <div class="collapse navbar-collapse" id="navbarNav">
      <ul class="navbar-nav">
        <li class="nav-item">
          <a class="nav-link active" href="#">Home</a>
        </li>
        <li class="nav-item">
          <a class="nav-link disabled" href="#images">Images</a>
        </li>
        <li class="nav-item">
          <a class="nav-link" href="#deleted">Deleted</a>
        </li>
      </ul>
    </div>
  </Navbar>
  <div class="d-flex min-vh-100 justify-content-center align-items-center">
    <form class="row g-3" @submit.prevent="navigate">
      <div class="col-auto">
        <input type="text" name="path" class="form-control form-control-lg" placeholder="Path" v-model="path"/>
      </div>
      <div class="col-auto">
        <button type="submit" class="btn btn-primary btn-lg mb-3" :disabled="!path">Open</button>
      </div>
    </form>
  </div>
</template>
<style scoped>
  .form-control-lg {
    width: 500px;
  }
</style>
