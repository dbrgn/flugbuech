<script>
  import {i18n} from '$lib/i18n';
</script>

<h2 class="title is-size-2">{$i18n.t('privacy-policy.title--privacy-policy', 'Privacy Policy')}</h2>

<h3 class="title is-size-3">Who is responsible for this website?</h3>

<p class="content is-italic">
  Bargen Software, Danilo Bargen<br />
  8732 Neuhaus SG, Switzerland<br />
  E-mail: <a href="mailto:flugbuech@bargen.dev">flugbuech@bargen.dev</a>
</p>

<h3 class="title is-size-3">General</h3>

<p class="content">
  This page explains what happens to your data when using this website, in accordance with article
  13 of the Swiss Federal Constitution and the data protection provisions of the Swiss Confederation
  (Data Protection Act, DSG).
</p>

<p class="content has-text-weight-bold">
  This website integrates map services from third party providers (Mapbox and Swisstopo). Apart from
  these map services, there are no third party trackers, no ads, no externally loaded fonts, or
  anything like that. Everything else loaded comes from the Flugbuech webserver.
</p>

<h3 class="title is-size-3">What data is being processed or collected?</h3>

<p class="content">
  When accessing this website, the following information is being collected by the web server and
  stored for 14 days in order to properly operate and optimize the website:
</p>

<table class="table is-hoverable">
  <thead>
    <tr>
      <th>Data point</th>
      <th>Example</th>
      <th>Comment</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Anonymized IP address</td>
      <td class="is-italic">178.197.222.0</td>
      <td>
        IP addresses are anonymized (by discarding the last octet) and stored in web server access
        logs for 14 days. After this period, the access logs are deleted.</td
      >
    </tr>
    <tr>
      <td>HTTP user agent</td>
      <td class="is-italic">
        Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0</td
      >
      <td>
        The browser's HTTP user agent is stored in web server access logs for 14 days. After this
        period, the access logs are deleted.</td
      >
    </tr>
  </tbody>
</table>

<p class="content">
  When signing up to this website, the following information is being collected and stored:
</p>

<table class="table is-hoverable">
  <thead>
    <tr>
      <th>Data point</th>
      <th>Comment</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>E-mail address</td>
      <td>
        Your e-mail address may be used by me to contact you if there are any issues with your
        account. The e-mail address is not published anywhere.
      </td>
    </tr>
  </tbody>
</table>

<p class="content">
  When submitting flights to this website as a registered user, the following information is being
  collected and stored (until deleted by you), because that is the whole purpose of this website:
</p>

<table class="table is-hoverable">
  <thead>
    <tr>
      <th>Data point</th>
      <th>Comment</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Information about flight</td>
      <td
        >Launch location, landing location, glider, flight duration, distance, etc. This information
        is manually entered or imported from an IGC file.
      </td>
    </tr>
    <tr>
      <td>IGC flight recording</td>
      <td
        >The IGC file (optional) includes timestamps and location information from the flight. It
        might also include the pilot's name.
      </td>
    </tr>
  </tbody>
</table>

<p class="content">
  For keeping track of usage statistics, a self-hosted instance of <a href="https://plausible.io/"
    >Plausible</a
  > is being used. The data is only used because I am curious how Flugbuech is being used, and want to
  be able to optimize the website. It is not being shared with any third party. Plausible does not collect
  any personal data or personal identifiable information.
</p>

<p class="content">
  When accessing this website, the following information is being collected by the access statistics
  tool (Plausible) and stored indefinitely:
</p>

<table class="table is-hoverable">
  <thead>
    <tr>
      <th>Data point</th>
      <th>Example</th>
      <th>Comment</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Page URL</td>
      <td class="is-italic">https://flugbuech.bargen.dev/stats/</td>
      <td>
        We track the page URL of each page view on our website, in order to determine how many times
        a specific pages has been viewed.</td
      >
    </tr>
    <tr>
      <td>HTTP referer</td>
      <td class="is-italic">https://duckduckgo.com/</td>
      <td>
        We use the referrer string to find the number of visitors referred to our website from links
        on other sites.</td
      >
    </tr>
    <tr>
      <td>Browser</td>
      <td class="is-italic">Firefox 120.0.1</td>
      <td>
        We use this to find out what browsers and browser version numbers people use when visiting
        our website. This is derived from the User-Agent HTTP header. The full User-Agent is
        discarded.</td
      >
    </tr>
    <tr>
      <td>Operating system</td>
      <td class="is-italic">macOS 10.15</td>
      <td>
        We use this to find out what operating systems people use when visiting our website. We
        store the brand of the operating system and the version number. This is derived from the
        User-Agent HTTP header. The full User-Agent is discarded.</td
      >
    </tr>
    <tr>
      <td>Device type</td>
      <td class="is-italic">Desktop</td>
      <td>
        We use this to find out what devices people use when visiting your website. Devices are
        categorized into desktop, mobile or tablet. This is derived from the User-Agent HTTP header.
        The full User-Agent is discarded.</td
      >
    </tr>
    <tr>
      <td>Country, region, city</td>
      <td class="is-italic">Switzerland, St. Gallen, Rapperswil</td>
      <td>
        We look up the visitor's location using their IP address. We do not track anything more
        granular than the city level and the IP address of the visitor is discarded.</td
      >
    </tr>
  </tbody>
</table>

<p class="content">
  More information about Plausible can be found at
  <a
    href="https://plausible.io/data-policy#first-thing-first-what-we-collect-and-what-we-use-it-for"
    >plausible.io/data-policy</a
  >.
</p>

<h4 class="title is-size-4">Third Party Map Services</h4>

<p class="content">
  This website uses map services from third party providers to display interactive maps. When you
  use the map functionality, your browser will connect to these external services to load map tiles
  and data.
</p>

<h5 class="title is-size-5">Mapbox</h5>

<p class="content">
  We use Mapbox for base map layers (Outdoors and Satellite views) and additional map data such as
  country boundaries and terrain information. When you view maps, your browser requests map tiles
  directly from Mapbox's servers.
</p>

<p class="content">Data processed by Mapbox may include:</p>

<div class="content">
  <ul>
    <li>Your IP address (for delivering map tiles)</li>
    <li>Requested map coordinates and zoom levels</li>
    <li>Browser information (User-Agent header)</li>
  </ul>
</div>

<p class="content">
  For more information about Mapbox's data handling, please see their privacy policy at
  <a href="https://www.mapbox.com/legal/privacy" target="_blank" rel="noopener noreferrer"
    >mapbox.com/legal/privacy</a
  >.
</p>

<h5 class="title is-size-5">Swisstopo</h5>

<p class="content">
  We use Swisstopo (Swiss Federal Office of Topography) for detailed Swiss topographic maps and
  aerial imagery (Swisstopo and Swissimage map types). When you view these map types, your browser
  requests map tiles directly from Swisstopo's servers.
</p>

<p class="content">Data processed by Swisstopo may include:</p>

<div class="content">
  <ul>
    <li>Your IP address (for delivering map tiles)</li>
    <li>Requested map coordinates and zoom levels</li>
    <li>Browser information (User-Agent header)</li>
  </ul>
</div>

<p class="content">
  For more information about Swisstopo's data handling, please see their privacy notice in the
  <a
    href="https://www.admin.ch/gov/en/start/terms-and-conditions.html"
    target="_blank"
    rel="noopener noreferrer">admin.ch Terms and Conditions</a
  >.
</p>

<h3 class="title is-size-3">Cookies</h3>

<p class="content">
  The website uses two persistent cookies for keeping you logged in (expiring after 1 year) and for
  remembering your user name.
</p>

<h3 class="title is-size-3">Who is this data shared with?</h3>

<p class="content">
  Your personal account and flight data (registration information, flight data) is not being shared
  with any third party. However, when using the map functionality, your browser will directly
  connect to third-party map service providers (Mapbox and Swisstopo) to load map tiles. This means
  these services may collect technical data as described in the "Third Party Map Services" section
  above.
</p>

<h3 class="title is-size-3">Where is the data stored / transferred to?</h3>

<p class="content">
  The data is stored on a virtual server located in Germany hosted by <a
    href="https://www.netcup.de/">Netcup</a
  > and operated by us.
</p>

<h3 class="title is-size-3">Your Rights</h3>

<div class="content">
  <ul>
    <li>
      You have the <strong>right of access</strong>: Send an e-mail to
      <a href="mailto:flugbuech@bargen.dev">flugbuech@bargen.dev</a> requesting your data, and I will
      send it to you.
    </li>
    <li>
      You have the <strong>right to erasure</strong>: Send an e-mail to
      <a href="mailto:flugbuech@bargen.dev">flugbuech@bargen.dev</a> requesting the deletion of your
      account and all your data, and I will comply (after a verification).
    </li>
  </ul>
</div>

<h3 class="title is-size-3">Last Update</h3>

<p class="content">This page was last updated on August 22, 2025.</p>
