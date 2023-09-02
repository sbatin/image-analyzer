const app = Vue.createApp({
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
    const url = new URL(document.location.href);
    return {
      path: url.searchParams.get('path'),
      images: [],
      progress: false,
      similarImages: undefined,
      distance: 10,
    }
  },
  mounted() {
    fetch(`/list_folder?path=${this.path}`)
      .then((resp) => resp.json())
      .then((images) => {
        this.images = images;
      })
  }
});

app.mount('#app');