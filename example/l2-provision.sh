#!/bin/bash

set -euxo pipefail

sudo apt-get update && sudo apt-get install -y  rsync php php8.1-cli php-xml

# install sysbench
if ! which sysbench > /dev/null ; then
    curl -s https://packagecloud.io/install/repositories/akopytov/sysbench/script.deb.sh | sudo bash
    sudo apt-get -y install sysbench
fi

# install phoronix-test-suite
if ! which phoronix-test-suite > /dev/null ; then
    wget http://phoronix-test-suite.com/releases/repo/pts.debian/files/phoronix-test-suite_10.8.4_all.deb
    sudo dpkg -i phoronix-test-suite_10.8.4_all.deb
    sudo apt-get install -f
    sudo apt --fix-broken install
fi
phoronix-test-suite install build-linux-kernel

# set phoronix-test-suite configuration
mkdir -p ~/.phoronix-test-suite
cat <<EOF > ~/.phoronix-test-suite/user-config.xml
<?xml version="1.0"?>
<!--Phoronix Test Suite v10.8.4-->
<PhoronixTestSuite>
  <Options>
    <OpenBenchmarking>
      <AnonymousUsageReporting>TRUE</AnonymousUsageReporting>
      <IndexCacheTTL>3</IndexCacheTTL>
      <AlwaysUploadSystemLogs>FALSE</AlwaysUploadSystemLogs>
      <AllowResultUploadsToOpenBenchmarking>TRUE</AllowResultUploadsToOpenBenchmarking>
    </OpenBenchmarking>
    <General>
      <DefaultBrowser></DefaultBrowser>
      <UsePhodeviCache>TRUE</UsePhodeviCache>
      <DefaultDisplayMode>DEFAULT</DefaultDisplayMode>
      <PhoromaticServers></PhoromaticServers>
      <ColoredConsole>AUTO</ColoredConsole>
    </General>
    <Modules>
      <AutoLoadModules>toggle_screensaver, update_checker, perf_tips, ob_auto_compare, load_dynamic_result_viewer</AutoLoadModules>
    </Modules>
    <Installation>
      <RemoveDownloadFiles>FALSE</RemoveDownloadFiles>
      <SearchMediaForCache>TRUE</SearchMediaForCache>
      <SymLinkFilesFromCache>FALSE</SymLinkFilesFromCache>
      <PromptForDownloadMirror>FALSE</PromptForDownloadMirror>
      <EnvironmentDirectory>~/.phoronix-test-suite/installed-tests/</EnvironmentDirectory>
      <CacheDirectory>~/.phoronix-test-suite/download-cache/</CacheDirectory>
    </Installation>
    <Testing>
      <SaveSystemLogs>TRUE</SaveSystemLogs>
      <SaveInstallationLogs>TRUE</SaveInstallationLogs>
      <SaveTestLogs>TRUE</SaveTestLogs>
      <RemoveTestInstallOnCompletion>FALSE</RemoveTestInstallOnCompletion>
      <ResultsDirectory>~/.phoronix-test-suite/test-results/</ResultsDirectory>
      <AlwaysUploadResultsToOpenBenchmarking>FALSE</AlwaysUploadResultsToOpenBenchmarking>
      <AutoSortRunQueue>TRUE</AutoSortRunQueue>
      <ShowPostRunStatistics>TRUE</ShowPostRunStatistics>
    </Testing>
    <TestResultValidation>
      <DynamicRunCount>TRUE</DynamicRunCount>
      <LimitDynamicToTestLength>20</LimitDynamicToTestLength>
      <StandardDeviationThreshold>2.5</StandardDeviationThreshold>
      <ExportResultsTo></ExportResultsTo>
      <MinimalTestTime>2</MinimalTestTime>
      <DropNoisyResults>FALSE</DropNoisyResults>
    </TestResultValidation>
    <ResultViewer>
      <WebPort>RANDOM</WebPort>
      <LimitAccessToLocalHost>TRUE</LimitAccessToLocalHost>
      <AccessKey></AccessKey>
      <AllowSavingResultChanges>TRUE</AllowSavingResultChanges>
      <AllowDeletingResults>TRUE</AllowDeletingResults>
    </ResultViewer>
    <BatchMode>
      <SaveResults>FALSE</SaveResults>
      <OpenBrowser>FALSE</OpenBrowser>
      <UploadResults>FALSE</UploadResults>
      <PromptForTestIdentifier>FALSE</PromptForTestIdentifier>
      <PromptForTestDescription>FALSE</PromptForTestDescription>
      <PromptSaveName>FALSE</PromptSaveName>
      <RunAllTestCombinations>TRUE</RunAllTestCombinations>
      <Configured>TRUE</Configured>
    </BatchMode>
    <Networking>
      <NoInternetCommunication>FALSE</NoInternetCommunication>
      <NoNetworkCommunication>FALSE</NoNetworkCommunication>
      <Timeout>20</Timeout>
      <ProxyAddress></ProxyAddress>
      <ProxyPort></ProxyPort>
      <ProxyUser></ProxyUser>
      <ProxyPassword></ProxyPassword>
    </Networking>
    <Server>
      <RemoteAccessPort>RANDOM</RemoteAccessPort>
      <Password></Password>
      <WebSocketPort>RANDOM</WebSocketPort>
      <AdvertiseServiceZeroConf>TRUE</AdvertiseServiceZeroConf>
      <AdvertiseServiceOpenBenchmarkRelay>TRUE</AdvertiseServiceOpenBenchmarkRelay>
      <PhoromaticStorage>~/.phoronix-test-suite/phoromatic/</PhoromaticStorage>
    </Server>
  </Options>
</PhoronixTestSuite>
EOF
