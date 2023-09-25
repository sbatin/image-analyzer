<script>
import ImageList from './ImageList.vue';
import Preview from './Preview.vue';
import Settings from './Settings.vue';
import Navbar from './Navbar.vue';
import API from './api';

const MODE_LIST = 'list';
const MODE_PENDING = 'pending';
const MODE_RESULT = 'result';

function groupImages(images) {
  const groups = {};

  for (const file of images) {
    const year = new Date(file.date).getFullYear();
    if (!groups[year]) {
      groups[year] = [];
    }

    groups[year].push(file);
  }

  return Object.entries(groups)
    .sort((a, b) => b[0] - a[0])
    .map(([key, items]) => {
      return {
        title: key,
        items,
      }
    });
}

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
            .sort((a, b) => b[0].date - a[0].date)
            .map((items, i) => {
              return {
                title: `Group ${i + 1} (${items.length} images)`,
                items,
              }
            })
          return;
        }
      }
    },

    async analyze(params) {
      this.mode = MODE_PENDING;
      this.progress = 0;

      try {
        const response = await API.analyze(this.path, params);
        console.log(response);
        //await this.analyzePoll();

        API.subscribe(response.taskId, (progress) => {
          this.progress = progress;
          if (this.progress === 100) {
            this.analyzePoll(response.taskId);
          }
        });
      } catch (err) {
        this.error = err;
        this.mode = MODE_LIST;
      }
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
      groups: undefined,
      mode: 0,
      error: undefined,
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
    API.listDir(this.path)
      .then((images) => {
        this.groups = groupImages(images);
        this.mode = MODE_LIST;
      })
      .catch((err) => {
        this.error = err;
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
    <div class="container-fluid py-5">
      <div v-if="error" class="alert alert-danger" role="alert">
        <h4 class="alert-heading">Error</h4>
        <p>{{ error.message }}</p>
      </div>
      <div v-if="isPending">
        <p class="h3" style="text-align: center">Analyzing...</p>
        <div class="progress mx-3" role="progressbar" style="height: 20px">
          <div class="progress-bar progress-bar-striped progress-bar-animated" :style="`width: ${progress}%`"></div>
        </div>
      </div>
      <div v-if="!isPending">
        <div class="row row-cols-auto img-group" v-for="group of groups">
          <div class="group-title">{{ group.title }}</div>
          <ImageList :images="group.items" @click="$refs.modal.open"/>
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
  border-top: 1px solid var(--bs-border-color);
  padding: 20px 0;
}
.breadcrumb-item a {
  color: white;
}
.row {
  padding: 0 40px;
}
.group-title {
  margin-left:-40px;
  width: 100%;
  margin-bottom: 20px;
  font-size: 1.5em;
  color: var(--bs-tertiary-color);
}
</style>