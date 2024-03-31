<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {formatDateTime} from '$lib/formatters';

  import {SubmitError, type SubmitErrorData} from '../../api';

  import {
    analyzeCsv,
    NO_ROW,
    type CsvAnalyzeResult,
    type CsvImportResult,
    defaultMessageGroup,
    NO_FIELD,
    importCsv,
  } from './api';

  let flashes: Flashes;

  // Form values
  let files: FileList | undefined;

  // Error handling
  let submitError: SubmitErrorData | undefined;

  // Upload state
  type UploadState =
    | {step: 1; kind: 'upload'}
    | {step: 2; kind: 'analyzed'; result: CsvAnalyzeResult}
    | {step: 3; kind: 'imported'; result: CsvImportResult};
  let state: UploadState = {step: 1, kind: 'upload'};

  async function submitFormAnalyze(): Promise<void> {
    console.log('Sending CSV to API for analysis');
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
  }

  async function submitFormImport(): Promise<void> {
    console.log('Sending CSV to API for import');
    if (files === undefined || files.length === 0) {
      console.error('Missing CSV file');
      return;
    }

    let importResult;
    try {
      importResult = await importCsv(files[0]);
    } catch (error) {
      if (error instanceof SubmitError) {
        submitError = error.data;
      } else {
        submitError = {type: 'api-error', message: `Unknown API error: ${error}`};
      }
      return;
    }

    state = {step: 3, kind: 'imported', result: importResult};
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

<h2 class="title is-2">Import Flights from CSV: Step {state.step}/3</h2>

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
        void submitFormAnalyze();
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
        <i class="fa-solid fa-danger" />&ensp;<strong>Errors:</strong>
        <ul>
          {#each state.result.errors as error}<li>
              {#if error.csvRow !== undefined}Row {error.csvRow}:
              {/if}{error.message}
            </li>{/each}
        </ul>
      </div>
    </article>
  {/if}
  {#if state.result.warnings.length > 0}
    <article class="message is-warning">
      <div class="message-body content">
        <i class="fa-solid fa-warning" />&ensp;<strong>Warnings:</strong>
        <ul>
          {#each state.result.warnings as warning}<li>
              {#if warning.csvRow !== undefined}Row {warning.csvRow}:
              {/if}{warning.message}
            </li>{/each}
        </ul>
      </div>
    </article>
  {/if}
  {#if state.result.errors.length === 0 && state.result.warnings.length === 0}
    <article class="message is-success">
      <div class="message-body content">
        Hooray, CSV file looks valid, no warnings or errors were detected. Click the
        <strong>Import CSV</strong> button below, to import {state.result.flights.length} flights into
        your flight book.<br />
      </div>
    </article>
  {/if}

  <div class="table-container">
    <table class="table is-fullwidth is-striped is-hoverable" style="font-size: 14px;">
      <thead>
        <tr>
          <th>Row</th>
          <th>#</th>
          <th>Glider</th>
          <th>Launch<br />Time</th>
          <th>Launch<br />Location</th>
          <th>Landing<br />Time</th>
          <th>Landing<br />Location</th>
          <th>GPS<br />Distance</th>
          <th>XContest</th>
          <th>Comment</th>
          <th>Video</th>
          <th>H&amp;F</th>
        </tr>
      </thead>
      <tbody>
        {#each state.result.flights as flight}
          {@const messages = state.result.messagesByRowAndField}

          {@const hasUnspecificErrors =
            messages[NO_ROW].errors.length > 0 ||
            messages[flight.csvRow]?.[NO_FIELD].errors.length > 0}
          {@const hasUnspecificWarnings =
            messages[NO_ROW].warnings.length > 0 ||
            messages[flight.csvRow]?.[NO_FIELD].warnings.length > 0}

          {@const numberMessages = messages[flight.csvRow]?.['number'] ?? defaultMessageGroup()}
          {@const gliderIdMessages =
            messages[flight.csvRow]?.['glider-id'] ?? defaultMessageGroup()}
          {@const launchTimeMessages =
            messages[flight.csvRow]?.['launch-time'] ?? defaultMessageGroup()}
          {@const launchAtMessages =
            messages[flight.csvRow]?.['launch-at'] ?? defaultMessageGroup()}
          {@const landingTimeMessages =
            messages[flight.csvRow]?.['landing-time'] ?? defaultMessageGroup()}
          {@const landingAtMessages =
            messages[flight.csvRow]?.['landing-at'] ?? defaultMessageGroup()}
          {@const xcontestTracktypeMessages =
            messages[flight.csvRow]?.['xcontest-tracktype'] ?? defaultMessageGroup()}
          {@const xcontestUrlMessages =
            messages[flight.csvRow]?.['xcontest-url'] ?? defaultMessageGroup()}

          <tr
            class:is-danger={hasUnspecificErrors}
            class:is-warning={!hasUnspecificErrors && hasUnspecificWarnings}
          >
            <td>{flight.csvRow}</td>
            <td
              class:is-danger={numberMessages.errors.length > 0}
              class:is-warning={numberMessages.warnings.length > 0}
              title={[...numberMessages.errors, ...numberMessages.warnings]
                .map((m) => m.message)
                .join('\n')}
            >
              {#if flight.number === undefined}
                {#if numberMessages.errors.length > 0}
                  <i class="fa-solid fa-danger" />
                {:else if numberMessages.warnings.length > 0}
                  <i class="fa-solid fa-warning" />
                {:else}
                  -
                {/if}
              {:else}
                {flight.number}
              {/if}
            </td>
            <td
              class:is-danger={gliderIdMessages.errors.length > 0}
              class:is-warning={gliderIdMessages.warnings.length > 0}
              title={[...gliderIdMessages.errors, ...gliderIdMessages.warnings]
                .map((m) => m.message)
                .join('\n')}
            >
              {#if flight.gliderId === undefined}
                {#if gliderIdMessages.errors.length > 0}
                  <i class="fa-solid fa-danger" />
                {:else if gliderIdMessages.warnings.length > 0}
                  <i class="fa-solid fa-warning" />
                {:else}
                  -
                {/if}
              {:else}
                <a href="/gliders/{flight.gliderId}/edit/" target="_blank">
                  {flight.gliderId}
                </a>
              {/if}
            </td>
            <td
              class:is-danger={launchTimeMessages.errors.length > 0}
              class:is-warning={launchTimeMessages.warnings.length > 0}
              title={[...launchTimeMessages.errors, ...launchTimeMessages.warnings]
                .map((m) => m.message)
                .join('\n')}
            >
              {#if flight.launchTime === undefined}
                {#if launchTimeMessages.errors.length > 0}
                  <i class="fa-solid fa-danger" />
                {:else if launchTimeMessages.warnings.length > 0}
                  <i class="fa-solid fa-warning" />
                {:else}
                  -
                {/if}
              {:else}
                {formatDateTime(flight.launchTime)}
              {/if}
            </td>
            <td
              class:is-danger={launchAtMessages.errors.length > 0}
              class:is-warning={launchAtMessages.warnings.length > 0}
              title={[...launchAtMessages.errors, ...launchAtMessages.warnings]
                .map((m) => m.message)
                .join('\n')}
            >
              {#if flight.launchAt === undefined}
                {#if launchAtMessages.errors.length > 0}
                  <i class="fa-solid fa-danger" />
                {:else if launchAtMessages.warnings.length > 0}
                  <i class="fa-solid fa-warning" />
                {:else}
                  -
                {/if}
              {:else}
                <a href="/locations/{flight.launchAt}/" target="_blank">
                  {flight.launchAt}
                </a>
              {/if}
            </td>
            <td
              class:is-danger={landingTimeMessages.errors.length > 0}
              class:is-warning={landingTimeMessages.warnings.length > 0}
              title={[...landingTimeMessages.errors, ...landingTimeMessages.warnings]
                .map((m) => m.message)
                .join('\n')}
            >
              {#if flight.landingTime === undefined}
                {#if landingTimeMessages.errors.length > 0}
                  <i class="fa-solid fa-danger" />
                {:else if landingTimeMessages.warnings.length > 0}
                  <i class="fa-solid fa-warning" />
                {:else}
                  -
                {/if}
              {:else}
                {formatDateTime(flight.landingTime)}
              {/if}
            </td>
            <td
              class:is-danger={landingAtMessages.errors.length > 0}
              class:is-warning={landingAtMessages.warnings.length > 0}
              title={[...landingAtMessages.errors, ...landingAtMessages.warnings]
                .map((m) => m.message)
                .join('\n')}
            >
              {#if flight.landingAt === undefined}
                {#if landingAtMessages.errors.length > 0}
                  <i class="fa-solid fa-danger" />
                {:else if landingAtMessages.warnings.length > 0}
                  <i class="fa-solid fa-warning" />
                {:else}
                  -
                {/if}
              {:else}
                <a href="/locations/{flight.landingAt}/" target="_blank">
                  {flight.landingAt}
                </a>
              {/if}
            </td>
            <td>{flight.trackDistance ?? '-'}</td>
            <td
              class:is-danger={xcontestTracktypeMessages.errors.length > 0 ||
                xcontestUrlMessages.errors.length > 0}
              class:is-warning={xcontestTracktypeMessages.warnings.length > 0 ||
                messages[flight.csvRow]?.['xcontest-url'].warnings.length > 0}
              title={[
                ...xcontestTracktypeMessages.errors,
                ...xcontestTracktypeMessages.warnings,
                ...xcontestUrlMessages.errors,
                ...xcontestUrlMessages.warnings,
              ]
                .map((m) => m.message)
                .join('\n')}
            >
              <em>Tracktype:</em>
              {#if flight.xcontestTracktype === undefined}
                {#if xcontestTracktypeMessages.errors.length > 0}
                  <i class="fa-solid fa-danger" />
                {:else if xcontestTracktypeMessages.warnings.length > 0}
                  <i class="fa-solid fa-warning" />
                {:else}
                  -
                {/if}
              {:else}
                {flight.xcontestTracktype}
              {/if}
              <br />
              <em>Distance:</em>
              {flight.xcontestDistance ?? '-'}<br />
              <em>URL:</em>
              {#if flight.xcontestUrl === undefined}
                {#if xcontestUrlMessages.errors.length > 0}
                  <i class="fa-solid fa-danger" />
                {:else if xcontestUrlMessages.warnings.length > 0}
                  <i class="fa-solid fa-warning" />
                {:else}
                  -
                {/if}
              {:else}
                <a href={flight.xcontestUrl} target="_blank" rel="noopener noreferrer">
                  {flight.xcontestUrl}
                </a>
              {/if}
            </td>
            <td class="preserve-newlines">{flight.comment ?? '-'}</td>
            <td>{flight.videoUrl ?? '-'}</td>
            <td>{flight.hikeandfly ? 'yes' : 'no'}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <div class="content submitbutton">
    <p>Do you want to import the flights above into your flight book?</p>
    <button
      class="button is-primary"
      disabled={state.result.errors.length > 0}
      type="button"
      on:click={submitFormImport}
    >
      Import CSV
    </button>
  </div>
{:else if state.kind === 'imported'}
  <article
    class="message"
    class:is-success={state.result.success}
    class:is-danger={!state.result.success}
  >
    <div class="message-body">
      {#if state.result.success}
        <i class="fa-solid fa-circle-check" />&nbsp;Successfully imported flights from CSV! Go to
        your <a href="/flights/">flight list</a> to see them.
      {:else}
        <strong>Error: </strong>Failed to import flights from CSV.
      {/if}
    </div>
  </article>
{/if}

<style>
  .example {
    font-size: 0.875em;
  }

  .submitbutton {
    margin-top: 2em;
  }
</style>
