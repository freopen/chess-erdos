<template>
  <q-page class="row items-center justify-evenly">
    <example-component
      title="Example component"
      active
      :todos="todos"
      :meta="meta"
    ></example-component>
    {{ result }}
  </q-page>
</template>

<script lang="ts">
import { Todo, Meta } from 'components/models';
import ExampleComponent from 'components/CompositionComponent.vue';
import { defineComponent, ref } from 'vue';
import gql from 'graphql-tag';
import { useQuery } from '@vue/apollo-composable';

export default defineComponent({
  name: 'PageIndex',
  components: { ExampleComponent },
  setup() {
    const { result } = useQuery(gql`
      query getUser {
        user(id: "anibalpg") {
          id
          erdosNumber
          erdosLinks {
            loserId
            erdosNumber
          }
        }
      }`);
    const todos = ref<Todo[]>([
      {
        id: 1,
        content: 'ct1',
      },
      {
        id: 2,
        content: 'ct2',
      },
      {
        id: 3,
        content: 'ct3',
      },
      {
        id: 4,
        content: 'ct4',
      },
      {
        id: 5,
        content: 'ct5',
      },
    ]);
    const meta = ref<Meta>({
      totalCount: 1200,
    });
    return { todos, meta, result };
  },
});
</script>
