<script>
  import Error from './Error.vue';
  import Navbar from './Navbar.vue';
  import API from './api';

  export default {
    methods: {
      getFileName(path) {
        const parts = path.split('/');
        return parts[parts.length - 1];
      },

      async restore() {
        if (this.selected) {
          try {
            await API.restoreFile(this.selected);
            this.selected = undefined;
          } catch (err) {
            this.error = err;
          }

          await this.refresh();
        }
      },

      async refresh() {
        try {
          this.items = await API.listDeleted(this.path);
        } catch (err) {
          this.error = err;
        }
      },
    },

    data() {
      return {
        items: [],
        selected: undefined,
        error: undefined,
      };
    },

    mounted() {
      this.refresh();
    },

    components: { Error, Navbar }
  }
</script>
<template>
  <Navbar>
    <div class="collapse navbar-collapse" id="navbarNav">
      <ul class="navbar-nav">
        <li class="nav-item">
          <a class="nav-link" href="#">Home</a>
        </li>
        <li class="nav-item">
          <a class="nav-link disabled" href="#images">Images</a>
        </li>
        <li class="nav-item">
          <a class="nav-link active" href="#deleted">Deleted</a>
        </li>
      </ul>
    </div>
    <button class="btn btn-success" type="button" @click="restore" :disabled="!selected">Restore</button>
  </Navbar>
  <div class="content" @click="selected = undefined">
    <div class="container-fluid py-5">
      <Error :error="error"/>
      <h4 class="display-4 text-secondary" v-if="items.length === 0">You don't have any removed files</h4>
      <div class="row row-cols-auto">
        <div class="col" v-for="file of items">
          <figure :class="{ figure, selected: file.id === selected}">
            <a href="javascript:void(0)" @click.stop.prevent="selected = file.id">
              <img class="figure-img img-fluid rounded" :src="`deleted/${file.id}`" :title="file.path"/>
            </a>
            <figcaption class="figure-caption img-title">{{ getFileName(file.path) }}</figcaption>
          </figure>
        </div>
      </div>
    </div>
  </div>
</template>
<style scoped>
.content {
  padding-top: 60px;
  height: 100vh;
}
.display-4 {
  text-align: center;
}
.row {
  padding: 0 40px;
}
.figure-img {
  max-height: 200px;
}
.selected .figure-img {
  border: 3px solid #0d6efd;
}
.img-title {
  max-width: 150px;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}
</style>
