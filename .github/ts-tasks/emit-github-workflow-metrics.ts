import core from "@actions/core";
import github from "@actions/github";
import * as fs from 'fs';

const [owner, repo] = process.env.GITHUB_REPOSITORY ? process.env.GITHUB_REPOSITORY.split("/") : ["aptos-labs", "aptos-core"]
const VICTORIAMETRICS_URL = process.env.VICTORIAMETRICS_URL;
if (!VICTORIAMETRICS_URL) {
    throw new Error("Missing environment variable `VICTORIAMETRICS_TOKEN`");
}
const VICTORIAMETRICS_TOKEN = process.env.VICTORIAMETRICS_TOKEN;
if (!VICTORIAMETRICS_TOKEN) {
    throw new Error("Missing environment variable `VICTORIAMETRICS_TOKEN`");
}

const WORKFLOW_METRICS_TO_EXPORT = process.env.WORKFLOW_METRICS_TO_EXPORT;
if (!WORKFLOW_METRICS_TO_EXPORT) {
    core.info("No workflow metrics to export. Exporting default ones only.");
}
const CUSTOM_METRICS_TO_EXPORT = fs.readFileSync(WORKFLOW_METRICS_TO_EXPORT!, "utf-8").split("\n").filter((line) => line.trim().length > 0);


export async function writeMetric(name: string, value: number, labels: Map<string, string>) {

    const response = await fetch(`${VICTORIAMETRICS_URL}/api/v1/import/prometheus`, {
        method: "POST",
        headers: {
            "Authorization": "Bearer " + VICTORIAMETRICS_TOKEN!
        },
        body: `${name}{${Array.from(labels.entries()).map(([k, v]) => `${k}="${v}"`).join(",")}} ${value}`
    });
    if (!response.ok) {
        core.setFailed(await response.text());
    }
}


// Export some calculated and default (https://docs.github.com/en/actions/learn-github-actions/variables#default-environment-variables) vars
export async function emitGithubWorkflowMetrics() {
    const githubToken = process.env.GITHUB_TOKEN;
    if (!githubToken) {
        throw new Error("Missing environment variable `GITHUB_TOKEN`");
    }
    const runId = process.env.GITHUB_RUN_ID;
    if (!runId) {
        throw new Error("Missing environment variable `GITHUB_RUN_ID`");
    }

    const ghClient = github.getOctokit(githubToken);
    const workflowDetails = await ghClient.rest.actions.getWorkflowRun({
        owner,
        repo,
        run_id: Number(runId),
    }).then((response) => response.data);

    if (!workflowDetails.run_started_at) {
        core.setFailed("Missing run_started_at");
        return;
    }
    const startDate = new Date(workflowDetails.run_started_at);
    const currDate = new Date();
    const diff = currDate.getTime() - startDate.getTime();
    const diffInMinutes = diff / (1000 * 60);
    core.info(`Workflow ran for ${diffInMinutes} minutes`);

    const workflow_git_sha = workflowDetails.head_sha!;
    const code_git_sha = workflowDetails.head_commit!.id;
    core.info(`Workflow git sha: ${workflow_git_sha}`);
    core.info(`Code git sha: ${code_git_sha}`);

    writeMetric("github_workflow_duration_minutes", diffInMinutes, new Map([["workflow_git_sha", workflow_git_sha], ["git_sha", code_git_sha]]));

}

// Run the function above.
emitGithubWorkflowMetrics();
