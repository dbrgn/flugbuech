<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';

  import {SubmitError, type SubmitErrorData} from '../../api';

  import {analyzeCsv} from './api';

  let flashes: Flashes;

  // Form values
  let files: FileList | undefined;

  // Error handling
  let submitError: SubmitErrorData | undefined;

  async function submitForm(): Promise<void> {
    console.log('Sending CSV to API');
    if (files === undefined || files.length === 0) {
      console.error('Missing CSV file');
      return;
    }

    let analyzeResult;
    try {
      analyzeResult = await analyzeCsv(files[0]);
    } catch (error) {
      if (error instanceof SubmitError) {
        submitError = error.data;
      } else {
        submitError = {type: 'api-error', message: `Unknown API error: ${error}`};
      }
      return;
    }

    console.log(analyzeResult); // TODO
  }

  $: hasFile = (files?.length ?? 0) > 0;
  $: fileName = files?.[0].name ?? 'No file selected…';
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/flights/">Flights</a></li>
    <li class="is-active"><a href="./" aria-current="page">Import from CSV</a></li>
  </ul>
</nav>

{#if submitError?.type === 'authentication'}
  <MessageModal
    type="warning"
    title="Authentication Error"
    message="Your login session has expired. Please log in again."
    showClose={false}
  >
    <section slot="buttons">
      <a href="/auth/login/?redirect=/flights/import/csv/" class="button is-warning">Login</a>
    </section>
  </MessageModal>
{:else if submitError?.type === 'api-error'}
  <MessageModal
    type="error"
    title="API Error"
    message="The CSV could not be processed due to an error on the server: {submitError.message}"
    showClose={true}
    on:closed={() => (submitError = undefined)}
  />
{/if}

<Flashes bind:this={flashes} />

<h2 class="title is-2">Import Flights from CSV</h2>

<article class="message is-warning">
  <div class="message-body">
    <i class="fa-solid fa-warning" />&ensp;Warning: This import is still experimental. If you
    experience any problems, please contact me at
    <a href="mailto:flugbuech@bargen.dev">flugbuech@bargen.dev</a>!
  </div>
</article>

<section>
  <div class="content">
    <p>You can import a list of flights from a CSV file. It needs to follow this format:</p>
    <ul>
      <li>First row contains header fields</li>
      <li>Character set: UTF-8</li>
      <li>Delimiter: Comma (<code>,</code>)</li>
      <li>Quoting: Double quotes (<code>"</code>)</li>
      <li>Max file size: 10 MiB</li>
    </ul>
    <details>
      <summary><em>Click to expand full CSV format description</em></summary>
      <h2>CSV format description</h2>
      <p>
        The following header fields are supported (but they are all optional). All fields may be
        omitted or empty (but at least one valid column must be present).
      </p>
      <table>
        <tr>
          <th>Field</th>
          <th>Type</th>
          <th>Description</th>
          <th>Example</th>
        </tr>
        <tr>
          <td><code>number</code></td>
          <td>Integer</td>
          <td>Your flight number</td>
          <td class="example"><code>108</code></td>
        </tr>
        <tr>
          <td><code>date</code></td>
          <td>ISO&nbsp;String</td>
          <td>The date as ISO string</td>
          <td class="example"><code>2024-03-17</code></td>
        </tr>
        <tr>
          <td><code>glider</code></td>
          <td>String</td>
          <td>
            The name of your glider/wing<br />
            <small>
              Must match the manufacturer and model of a glider you have already added to Flugbuech.
            </small>
          </td>
          <td class="example"><code>Advance Xi 21</code></td>
        </tr>
        <tr>
          <td><code>launch_site</code></td>
          <td>String</td>
          <td>
            The name of the launch site<br />
            <small>Must match the name of a location you have already added to Flugbuech.</small>
          </td>
          <td class="example"><code>Ebenalp</code></td>
        </tr>
        <tr>
          <td><code>launch_time_utc</code></td>
          <td>ISO&nbsp;String</td>
          <td>The launch time (UTC!) as ISO time string (including seconds)</td>
          <td class="example"><code>13:37:00</code></td>
        </tr>
        <tr>
          <td><code>landing_site</code></td>
          <td>String</td>
          <td>
            The name of the landing site<br />
            <small>Must match the name of a location you have already added to Flugbuech.</small>
          </td>
          <td class="example"><code>Wasserauen</code></td>
        </tr>
        <tr>
          <td><code>landing_time_utc</code></td>
          <td>ISO&nbsp;String</td>
          <td>The landing time (UTC!) as ISO time string (including seconds)</td>
          <td class="example"><code>15:42:23</code></td>
        </tr>
        <tr>
          <td><code>track_distance</code></td>
          <td>Float</td>
          <td>The GPS track distance (in km) of your flight</td>
          <td class="example"><code>37.86</code></td>
        </tr>
        <tr>
          <td><code>hikeandfly</code></td>
          <td>Boolean</td>
          <td>Was this a Hike&Fly? Either <code>true</code> or <code>false</code>.</td>
          <td class="example"><code>true</code></td>
        </tr>
        <tr>
          <td><code>comment</code></td>
          <td>String</td>
          <td>A comment about your flight</td>
          <td class="example"><code>Windy conditions, landed early</code></td>
        </tr>
        <tr>
          <td><code>xcontest_url</code></td>
          <td>String</td>
          <td>Link to your flight on XContest</td>
          <td class="example">
            <code>https://www.xcontest.org/world/en/flights/detail:dbrgn/7.3.2024/11:09</code>
          </td>
        </tr>
        <tr>
          <td><code>xcontest_tracktype</code></td>
          <td>String</td>
          <td>
            The track type of your flight on XContest<br />
            <small>
              Must be either <code>free_flight</code>, <code>flat_triangle</code> or
              <code>fai_triangle</code>.
            </small>
          </td>
          <td class="example"><code>flat_triangle</code></td>
        </tr>
        <tr>
          <td><code>xcontest_scored_distance</code></td>
          <td>Float</td>
          <td>The scored distance according to XContest</td>
          <td class="example"><code>36.79</code></td>
        </tr>
        <tr>
          <td><code>video_url</code></td>
          <td>String</td>
          <td>Link to a video of your flight</td>
          <td class="example"><code>https://www.youtube.com/watch?v=PgyNx0V-hsU</code></td>
        </tr>
      </table>
      <p>You can find an example CSV file <a href="/example.csv">here</a>.</p>
    </details>
  </div>
  <form
    method="post"
    on:submit={(event) => {
      event.preventDefault();
      void submitForm();
    }}
  >
    <div class="field">
      <div class="file has-name">
        <label class="file-label">
          <input id="igcFile" type="file" class="file-input" accept=".csv" bind:files />
          <span class="file-cta">
            <span class="file-icon">
              <i class="fa-solid fa-upload"></i>
            </span>
            <span class="file-label"> Click to upload CSV file </span>
          </span>
          <span class="file-name">{fileName}</span>
        </label>
      </div>
    </div>
    <div class="content submitbutton">
      <button class="button is-primary" disabled={!hasFile} type="submit"> Upload CSV </button>
    </div>
  </form>
</section>

<style>
  .example {
    font-size: 0.875em;
  }

  .submitbutton {
    margin-top: 2em;
  }
</style>
