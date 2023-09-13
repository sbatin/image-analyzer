<script>
import ImageList from './ImageList.vue';
import Preview from './Preview.vue';
import API from './api';

const MODE_LIST = 'list';
const MODE_PENDING = 'pending';
const MODE_RESULT = 'result';

export default {
  methods: {
    async analyzePoll(taskId) {
      const resp = await API.poll(taskId);
      switch (resp.type) {
        case 'Pending': {
          this.progress = resp.progress;
          await this.analyzePoll(taskId);
          break;
        }
        case 'Completed': {
          this.mode = MODE_RESULT;
          this.images = resp.data;
          return;
        }
      }
    },
    async analyze() {
      this.mode = MODE_PENDING;
      this.progress = 0;
      const response = await API.analyze(this.path, this.distance);
      console.log(response);
      //await this.analyzePoll();

      API.subscribe(response.taskId, (progress) => {
        this.progress = progress;
        if (this.progress === 100) {
          this.analyzePoll(response.taskId);
        }
      });
    },
  },
  data() {
    const [, search] = window.location.hash.split('?');
    const params = new URLSearchParams(search);
    return {
      path: params.get('path'),
      progress: 0,
      distance: 10,
      images: undefined,
      mode: 0,
    };
  },
  computed: {
    isList() {
      return this.mode === MODE_LIST;
    },
    isPending() {
      return this.mode === MODE_PENDING;
    },
    isReady() {
      return this.mode === MODE_RESULT;
    }
  },
  mounted() {
    API.listDir(this.path).then((images) => {
      this.images = images;
      this.mode = MODE_LIST;
    });
  },
  components: { ImageList, Preview }
}
</script>
<template>
  <nav class="navbar fixed-top bg-dark bg-body-tertiary" data-bs-theme="dark">
    <div class="container-fluid">
      <a class="navbar-brand" href="#">Image DeDup</a>
      <span class="navbar-text">{{ path }}</span>
      <ul class="navbar-nav me-auto mb-2 mb-lg-0"></ul>
      <input type="text" class="form-control me-2" style="max-width:100px" v-model="distance"/>
      <button class="btn btn-success" type="button" @click="analyze" :disabled="isPending">Analyze</button>
    </div>
  </nav>
  <div class="content">
    <div class="container py-5">
      <div v-if="isPending">
        <p class="h3" style="text-align: center">Analyzing...</p>
        <div class="progress mx-3" role="progressbar" style="height: 20px">
          <div class="progress-bar progress-bar-striped progress-bar-animated" :style="`width: ${progress}%`"></div>
        </div>
      </div>
      <div v-if="isList" class="row row-cols-auto gy-4">
        <ImageList :images="images" @click="$refs.modal.open"/>
      </div>
      <div v-if="isReady">
        <div class="row row-cols-auto img-group gy-4" v-for="group of images">
          <ImageList :images="group" @click="$refs.modal.open"/>
        </div>
      </div>
    </div>
  </div>
  <Preview ref="modal" :path="path"/>
</template>
<style scoped>
.content {
  padding-top: 60px;
}
.img-group {
  border-bottom: 1px solid var(--bs-border-color);
  padding: 20px 0;
}
</style>