<script>
  import Error from './Error.vue';
  import ImageList from './ImageList.vue';
  import Preview from './Preview.vue';
  import Settings from './Settings.vue';
  import Navbar from './Navbar.vue';
  import API from './api';
  import * as bootstrap from 'bootstrap';

  class Mode {
    static UNKNOWN = 0;
    static LIST = 1;
    static PENDING = 2;
    static READY = 3;
  }

  export default {
    methods: {
      addRelativePath(file) {
        const n = this.path.length;
        file.relativePath = file.path.substring(n + 1);
        return file;
      },

      groupImages(images) {
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
          .map(([title, files]) => {
            const items = files
              .map((file) => this.addRelativePath(file))
              .sort((a, b) => b.date - a.date);

            return { title, items };
          });
      },

      processGroups(groups) {
        return groups
          .map((group) => group.sort((a, b) => b.date - a.date))
          .sort((a, b) => b[0].date - a[0].date)
          .map((files, i) => {
            const items = files.map((file) => this.addRelativePath(file));
            return {
              title: `Group ${i + 1} (${items.length} images)`,
              items,
            }
          });
      },

      async analyzePoll(taskId) {
        const resp = await API.poll(taskId);
        switch (resp.type) {
          case 'Pending': {
            this.progress = resp.progress;
            return this.analyzePoll(taskId);
          }
          case 'Completed': {
            this.mode = Mode.READY;
            this.groups = this.processGroups(resp.data);
            return;
          }
        }
      },

      async analyze(params) {
        this.mode = Mode.PENDING;
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
          this.mode = Mode.LIST;
        }
      },

      async refresh() {
        try {
          const images = await API.listDir(this.path)
          this.groups = this.groupImages(images);
          this.mode = Mode.LIST;
        } catch (err) {
          this.error = err;
        }
      },
    },

    data() {
      const [, search] = window.location.hash.split('?');
      const params = new URLSearchParams(search);
      const path = params.get('path');
      return {
        path,
        progress: 0,
        groups: [],
        mode: Mode.UNKNOWN,
        error: undefined,
      };
    },

    computed: {
      isList() {
        return this.mode === Mode.LIST;
      },
      isPending() {
        return this.mode === Mode.PENDING;
      },
      isReady() {
        return this.mode === Mode.READY;
      }
    },

    mounted() {
      this.refresh();
    },

    components: { Error, ImageList, Preview, Settings, Navbar }
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
          <a class="nav-link active" :href="`#images?path=${path}`">Images</a>
        </li>
        <li class="nav-item">
          <a class="nav-link" href="#deleted">Deleted</a>
        </li>
      </ul>
    </div>
    <button class="btn btn-outline-light" type="button" onclick="window.location.reload(true)" :disabled="isList">Show all</button>
    <span style="width:10px"/>
    <button class="btn btn-success" type="button" @click="$refs.settings.open" :disabled="isPending">Analyze</button>
  </Navbar>
  <div class="content">
    <Preview ref="preview"/>
    <div class="container-fluid py-5">
      <Error :error="error"/>
      <div v-if="isPending">
        <p class="h3" style="text-align: center">Analyzing...</p>
        <div class="progress mx-3" role="progressbar" style="height: 20px">
          <div class="progress-bar progress-bar-striped progress-bar-animated" :style="`width: ${progress}%`"></div>
        </div>
      </div>
      <div v-if="isList || isReady">
        <div class="row row-cols-auto img-group" v-for="group of groups">
          <div class="group-title">{{ group.title }}</div>
          <ImageList :files="group.items" @click="(path) => $refs.preview.show(group.items, path)"/>
        </div>
      </div>
    </div>
  </div>
  <Settings ref="settings" @submit="analyze"/>
</template>
<style scoped>
.content {
  padding-top: 56px;
}
.img-group {
  border-top: 1px solid var(--bs-border-color);
  padding: 20px 0;
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
