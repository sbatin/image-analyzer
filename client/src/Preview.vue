<script>
  import * as bootstrap from 'bootstrap';
  import API from './api';
  import utils from './utils.js';

  export default {
    data() {
      return {
        files: [],
        active: 0,
        selected: 0,
        modal: undefined,
        hide_bs_modal: 'hide.bs.modal',
        slide_bs_carousel: 'slide.bs.carousel',
      }
    },

    computed: {
      selectedFile() {
        return this.files[this.selected];
      }
    },

    methods: {
      show(files, path) {
        this.files = files;
        this.active = this.files.findIndex((file) => file.path === path);
        this.selected = this.active;
        this.modal.show();
      },

      formatFile(file) {
        const size = utils.formatSize(file.size);
        const date = utils.formatDate(file.date);
        return `${date} (${size})`;
      },

      async deleteFile() {
        const path = this.selectedFile?.path;

        if (!confirm(`Are you sure you want to delete ${path}?`)) {
          return;
        }
        try {
          await API.deleteFile(path);
          console.log('deleted', path);

          this.files.splice(this.selected, 1);
          if (this.files.length === 0) {
            this.modal.hide();
          } else {
            this.active = this.selected;
          }
        } catch (err) {
          console.error(err);
        }
      },

      onHide(event) {
        this.active = 0;
        this.selected = 0;
        this.files = [];
      },
    },

    mounted() {
      this.modal = new bootstrap.Modal(this.$refs.modal);
    },
  }
</script>
<template>
  <div ref="modal" class="modal" tabindex="-1" @[hide_bs_modal]="onHide">
    <div class="modal-dialog modal-fullscreen">
      <div class="modal-content">
      <nav class="navbar bg-body-tertiary" data-bs-theme="dark">
        <div class="container-fluid">
          <span class="navbar-brand">
            <img src="/public/favicon.ico" alt="Logo" width="30" height="24" class="d-inline-block align-text-top">
            Image Analyzer
          </span>
          <span class="navbar-text">{{ selectedFile?.relativePath }}</span>
          <div>
            <button class="btn btn-outline-danger mx-2" type="button" @click="deleteFile">
              <i class="bi bi-trash"></i>
              Delete
            </button>
            <button class="btn btn-outline-secondary" type="button" data-bs-dismiss="modal">
              <i class="bi bi-x-lg"></i>
            </button>
          </div>
        </div>
      </nav>
      <div id="pv-mod-images" class="carousel slide carousel-fade" @[slide_bs_carousel]="(event) => selected = event.to">
        <div class="carousel-indicators">
          <button v-for="(_, i) in files" type="button" data-bs-target="#pv-mod-images" :data-bs-slide-to="i" :class="{active: i === active}"></button>
        </div>
        <div class="carousel-inner">
          <div v-for="(file, i) in files" :class="['carousel-item', i === active ? 'active' : '']">
            <div class="d-flex justify-content-center">
              <img :src="`image?path=${file.path}`" class="d-block"/>
              <div class="carousel-caption d-none d-md-block">
                <h5>{{ i + 1 }} / {{ files.length }}</h5>
                <p>{{ formatFile(file) }}</p>
              </div>
            </div>
          </div>
        </div>
        <button class="carousel-control-prev" type="button" data-bs-target="#pv-mod-images" data-bs-slide="prev">
          <span class="carousel-control-prev-icon"></span>
          <span class="visually-hidden">Previous</span>
        </button>
        <button class="carousel-control-next" type="button" data-bs-target="#pv-mod-images" data-bs-slide="next">
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
