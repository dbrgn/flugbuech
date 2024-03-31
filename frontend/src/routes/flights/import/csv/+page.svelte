<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {formatDateTime} from '$lib/formatters';

  import {SubmitError, type SubmitErrorData} from '../../api';

  import {analyzeCsv, type CsvAnalyzeResult} from './api';

  let flashes: Flashes;

  // Form values
  let files: FileList | undefined;

  // Error handling
  let submitError: SubmitErrorData | undefined;

  // Upload state
  type UploadState =
    | {step: 1; kind: 'upload'}
    | {step: 2; kind: 'analyzed'; result: CsvAnalyzeResult};
  let state: UploadState = {step: 1, kind: 'upload'};

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

    state = {step: 2, kind: 'analyzed', result: analyzeResult};
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

<h2 class="title is-2">Import Flights from CSV: Step {state.step}</h2>

<article class="message is-warning">
  <div class="message-body">
    <i class="fa-solid fa-warning" />&ensp;Warning: This import is still experimental. If you
    experience any problems, please contact me at
    <a href="mailto:flugbuech@bargen.dev">flugbuech@bargen.dev</a>!
  </div>
</article>

{#if state.kind === 'upload'}
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
                Must match the manufacturer and model of a glider you have already added to
                Flugbuech.
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
{:else if state.kind === 'analyzed'}
  <h3 class="title is-3">CSV Preview</h3>

  {#if state.result.errors.length > 0}
    <article class="message is-danger">
      <div class="message-body">
        <i class="fa-solid fa-danger" />&ensp;<strong>Error:</strong>
        <ul>
          {#each state.result.errors as error}<li>
              {#if error.csvRow !== null}Row {error.csvRow}:
              {/if}{error.message}
            </li>{/each}
        </ul>
      </div>
    </article>
  {/if}
  {#if state.result.warnings.length > 0}
    <article class="message is-warning">
      <div class="message-body content">
        <i class="fa-solid fa-warning" />&ensp;<strong>Warning:</strong>
        <ul>
          {#each state.result.warnings as warning}<li>
              {#if warning.csvRow !== null}Row {warning.csvRow}:
              {/if}{warning.message}
            </li>{/each}
        </ul>
      </div>
    </article>
  {/if}

  <table class="table is-fullwidth is-striped is-hoverable">
    <thead>
      <tr>
        <th>Row</th>
        <th>#</th>
        <th>Glider</th>
        <th>Launch Time</th>
        <th>Launch Location ID</th>
        <th>Landing Time</th>
        <th>Landing Location ID</th>
        <th>GPS Distance</th>
        <th>XContest</th>
        <th>Comment</th>
        <th>Video</th>
        <th>Hike&amp;Fly</th>
      </tr>
    </thead>
    <tbody>
      {#each state.result.flights as flight}
        {@const hasErrors = state.result.errors.some((error) => error.csvRow === flight.csvRow)}
        {@const hasWarnings = state.result.warnings.some((error) => error.csvRow === flight.csvRow)}
        <tr class:is-danger={hasErrors} class:is-warning={!hasErrors && hasWarnings}>
          <td>
            {flight.csvRow}
            {#if hasErrors}
              <i class="fa-solid fa-danger" />
            {:else if hasWarnings}
              <i class="fa-solid fa-warning" />
            {/if}
          </td>
          <td>{flight.number ?? '-'}</td>
          <td>
            {#if flight.gliderId === undefined}
              -
            {:else}
              <a href="/gliders/{flight.gliderId}/edit/" target="_blank">
                {flight.gliderId}
              </a>
            {/if}
          </td>
          <td>{flight.launchTime === undefined ? '-' : formatDateTime(flight.launchTime)}</td>
          <td>{flight.launchAt ?? '-'}</td>
          <td>{flight.landingTime === undefined ? '-' : formatDateTime(flight.landingTime)}</td>
          <td>{flight.landingAt ?? '-'}</td>
          <td>{flight.trackDistance ?? '-'}</td>
          <td>{flight.xcontestTracktype} / {flight.xcontestDistance} / {flight.xcontestUrl}</td>
          <td>{flight.comment ?? '-'}</td>
          <td>{flight.videoUrl ?? '-'}</td>
          <td>{flight.hikeandfly ? 'yes' : 'no'}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  .example {
    font-size: 0.875em;
  }

  .submitbutton {
    margin-top: 2em;
  }
</style>
