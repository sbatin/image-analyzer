<script>
import Home from './Home.vue';
import Images from './Images.vue';

const routes = {
  '/': Home,
  '/images': Images,
}

export default {
  data() {
    return {
      currentPath: window.location.hash,
    }
  },

  computed: {
    currentView() {
      const path = this.currentPath.replace('#', '/') || '/';
      const [name] = path.split('?');
      console.log('navigate', {path, name});
      return routes[name];
    }
  },

  mounted() {
    window.addEventListener('hashchange', () => {
		  this.currentPath = window.location.hash;
		});
  }
}
</script>

<template>
  <component :is="currentView"/>
</template>