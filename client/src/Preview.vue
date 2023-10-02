<script>
  import * as bootstrap from 'bootstrap';
  import API from './api';
  import utils from './utils.js';

  export default {
    data() {
      return {
        files: [],
        active: 0,
        selected: undefined,
        modal: undefined,
        title: '',
      }
    },

    methods: {
      show(group, selected) {
        this.selected = selected;
        this.files = group.items;
        this.title = group.title;
        this.active = this.files.findIndex((file) => file.path === selected);
        this.modal.show();
      },

      formatFile(file) {
        const size = utils.formatSize(file.size);
        const date = utils.formatDate(file.date);
        return `${date} (${size})`;
      },

      getFileName(path) {
        return utils.getFileName(path);
      },

      async deleteFile() {
        try {
          await API.deleteFile(this.selected);
          console.log('deleted', this.selected);

          const index = this.files.findIndex((file) => file.path === this.selected);
          if (index >= 0) {
            this.files.splice(index, 1);
          }

          if (this.files.length === 0) {
            this.modal.hide();
          } else {
            this.active = index;
            this.selected = undefined;
          }
        } catch (err) {
          console.error(err);
        }
      }
    },

    mounted() {
      this.modal = new bootstrap.Modal('#preview-modal');

      this.$refs.modal.addEventListener('hide.bs.modal', (event) => {
        this.active = 0;
        this.selected = undefined;
        this.files = [];
      });

      this.$refs.carousel.addEventListener('slide.bs.carousel', (event) => {
        this.selected = this.files[event.to].path;
      });
    },
  }
</script>
<template>
  <div id="preview-modal" ref="modal" class="modal" tabindex="-1">
    <div class="modal-dialog modal-fullscreen">
      <div class="modal-content">
      <nav class="navbar bg-body-tertiary" data-bs-theme="dark">
        <div class="container-fluid">
          <span class="navbar-brand">
            <img src="/public/favicon.ico" alt="Logo" width="30" height="24" class="d-inline-block align-text-top">
            Image DeDup
          </span>
          <span class="navbar-text">{{ title }}</span>
          <div>
            <button class="btn btn-outline-danger mx-2" type="button" @click="deleteFile">
              <i class="bi bi-trash"></i>
              Delete
            </button>
            <button class="btn btn-outline-light" type="button" data-bs-dismiss="modal">
              <i class="bi bi-x-lg"></i>
            </button>
            <!--button type="button" class="btn-close" data-bs-dismiss="modal"></button-->
          </div>
        </div>
      </nav>
      <div id="pv-images" ref="carousel" class="carousel slide carousel-fade">
        <div class="carousel-indicators">
          <button v-for="(file, index) in files" type="button" data-bs-target="#pv-images" :data-bs-slide-to="index" :class="{active: index === active}"></button>
        </div>
        <div class="carousel-inner">
          <div v-for="(file, index) in files" :class="['carousel-item', index === active ? 'active' : '']">
            <div class="d-flex justify-content-center">
              <img :src="`image?path=${file.path}`" class="d-block" :title="file.path"/>
              <div class="carousel-caption d-none d-md-block">
                <h5>{{ getFileName(file.path) }}</h5>
                <p>{{ formatFile(file) }}</p>
              </div>
            </div>
          </div>
        </div>
        <button class="carousel-control-prev" type="button" data-bs-target="#pv-images" data-bs-slide="prev">
          <span class="carousel-control-prev-icon"></span>
          <span class="visually-hidden">Previous</span>
        </button>
        <button class="carousel-control-next" type="button" data-bs-target="#pv-images" data-bs-slide="next">
          <span class="carousel-control-next-icon"></span>
          <span class="visually-hidden">Next</span>
        </button>
      </div>
      </div>
    </div>
  </div>
</template>
<style scoped>
  .modal-header {
    background-color: silver;
  }
  .carousel-inner {
    background-color: var(--bs-secondary-color);
  }
  .carousel img {
    height: calc(100vh - 56px);
  }
</style>
