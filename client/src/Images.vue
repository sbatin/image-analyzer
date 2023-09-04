<script>
export default {
  methods: {
    analyze() {
      this.progress = true;
      fetch(`/analyze?path=${this.path}&dist=${this.distance}`, {
        method: 'POST',
      })
        .then((resp) => resp.json())
        .then((data) => {
          console.log('similar images', data);
          this.progress = false;
          this.similarImages = data;
        })
    }
  },

  data() {
    const [, search] = window.location.hash.split('?');
    const params = new URLSearchParams(search);

    return {
      path: params.get('path'),
      progress: false,
      distance: 10,
      images: [],
      similarImages: undefined,
    }
  },

  mounted() {
    fetch(`/list_folder?path=${this.path}`)
      .then((resp) => resp.json())
      .then((images) => {
        this.images = images;
      })
  }
}
</script>

<template>
  <nav class="navbar fixed-top bg-dark bg-body-tertiary" data-bs-theme="dark">
    <div class="container-fluid">
      <a class="navbar-brand" href="#">Image DeDup</a>
      <span class="navbar-text">{{ path }}</span>
      <ul class="navbar-nav me-auto mb-2 mb-lg-0"></ul>
      <input type="text" class="form-control me-2" style="max-width:100px" v-model="distance"/>
      <button class="btn btn-success" type="button" @click="analyze" :disabled="progress">
        <span v-if="progress" class="spinner-border spinner-border-sm" aria-hidden="true"></span>
        <span v-if="progress" role="status">Analyzing...</span>
        <span v-if="!progress">Analyze</span>
      </button>
    </div>
  </nav>
  <div class="container py-5">
    <div v-if="!similarImages" class="row row-cols-auto gy-4">
      <div class="col" v-for="src of images">
        <img class="img-thumbnail" :src="`image?path=${src}`" :title="src"/>
      </div>
    </div>
    <div v-if="similarImages">
      <div class="row row-cols-auto img-group gy-4" v-for="group of similarImages">
        <span class="col" v-for="src of group">
          <img class="img-thumbnail" :src="`image?path=${src}`" :title="src"/>
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.img-thumbnail {
  max-height: 200px;
}
.img-group {
  border-bottom: 1px solid var(--bs-border-color);
  padding: 20px 0;
}
</style>