<script>
  import * as bootstrap from 'bootstrap';

  export default {
    emits: ['submit'],

    data() {
      return {
        modal: undefined,
        hashType: 'PHash',
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
          <h5 class="modal-title">Ananlyze Settings</h5>
          <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
        </div>
        <div class="modal-body">
          <div class="mb-3">
            <label for="hashType" class="form-label">Hash type</label>
            <select id="hashType" class="form-select" v-model="hashType">
              <option value="AHash">aHash</option>
              <option value="PHash">pHash</option>
              <option value="DHash">dHash</option>
            </select>
          </div>
          <div class="mb-3">
            <label for="hashSize" class="form-label">Hash size</label>
            <input type="text" id="hashSize" class="form-control" v-model="hashSize"/>
          </div>
          <div class="mb-3">
            <label for="distance" class="form-label">Max distance</label>
            <input type="text" id="hashSize" class="form-control" v-model="distance"/>
          </div>
        </div>
        <div class="modal-footer">
          <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Close</button>
          <button type="button" class="btn btn-primary" @click="submit">Submit</button>
        </div>
      </div>
    </div>
  </div>
</template>