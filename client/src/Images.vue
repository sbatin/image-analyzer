<script>
import ImageList from './ImageList.vue';
import Preview from './Preview.vue';
import Settings from './Settings.vue';
import Navbar from './Navbar.vue';
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
          this.groups = resp.data
            .map((group) => group.sort((a, b) => b.date - a.date))
            .sort((a, b) => b[0].date - a[0].date);
          return;
        }
      }
    },

    async analyze(params) {
      this.mode = MODE_PENDING;
      this.progress = 0;
      const response = await API.analyze(this.path, params);
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
    const path = params.get('path');
    const items = path.split('/');
    return {
      path,
      name: items[items.length - 1],
      progress: 0,
      images: undefined,
      groups: undefined,
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

  components: { ImageList, Preview, Settings, Navbar }
}
</script>
<template>
  <Navbar>
    <ol class="breadcrumb" style="margin: 0;">
      <li class="breadcrumb-item"><a href="#">Home</a></li>
      <li v-if="isList" class="breadcrumb-item active">{{ name }}</li>
      <li v-if="isReady || isPending" class="breadcrumb-item">
        <a :href="`#images?path=${path}`" onclick="window.location.reload(true)">
          {{ name }}
        </a>
      </li>
      <li v-if="isReady" class="breadcrumb-item active">Analyzed</li>
    </ol>
    <button class="btn btn-success" type="button" @click="$refs.settings.open" :disabled="isPending">Analyze</button>
  </Navbar>
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
        <div class="row row-cols-auto img-group gy-4" v-for="group of groups">
          <ImageList :images="group" @click="$refs.modal.open"/>
        </div>
      </div>
    </div>
  </div>
  <Preview ref="modal" :path="path"/>
  <Settings ref="settings" @submit="analyze"/>
</template>
<style scoped>
.content {
  padding-top: 60px;
}
.img-group {
  border-bottom: 1px solid var(--bs-border-color);
  padding: 20px 0;
}
.breadcrumb-item a {
  color: white;
}
</style>