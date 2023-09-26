<script>
  import * as bootstrap from 'bootstrap';

  export default {
    emits: ['submit'],

    data() {
      return {
        modal: undefined,
        hashType: 'DHash',
        hashSize: 8,
        distance: 5,
      }
    },

    methods: {
      open() {
        this.modal.show();
      },

      submit() {
        this.$emit('submit', {
          hashType: this.hashType,
          hashSize: this.hashSize,
          distance: this.distance,
        });
        this.modal.hide();
      }
    },

    mounted() {
      this.modal = new bootstrap.Modal('#settings-popup');
    },
  }
</script>
<template>
  <div id="settings-popup" class="modal" tabindex="-1">
    <div class="modal-dialog">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">Image processing options</h5>
          <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
        </div>
        <div class="modal-body">
          <div class="mb-3">
            <label for="hashType" class="form-label">Algorithm</label>
            <select id="hashType" class="form-select" v-model="hashType">
              <option value="AHash">aHash</option>
              <option value="PHash">pHash</option>
              <option value="DHash">dHash</option>
            </select>
          </div>
          <div class="mb-3">
            <label for="hashSize" class="form-label">Hash size ({{ hashSize }})</label>
            <input type="range" id="hashSize" class="form-range" min="8" max="16" v-model="hashSize"/>
          </div>
          <div class="mb-3">
            <label for="distance" class="form-label">Max distance ({{ distance }})</label>
            <input type="range" id="hashSize" class="form-range" v-model="distance"/>
          </div>
        </div>
        <div class="modal-footer">
          <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Close</button>
          <button type="submit" class="btn btn-primary" @click="submit">Submit</button>
        </div>
      </div>
    </div>
  </div>
</template>