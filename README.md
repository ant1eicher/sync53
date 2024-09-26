# sync53
A small CLI tool to update a Route53 DNS record to point to the WAN ip address of the host it is running on. Can be used periodically as a cron job.

## Usage

```
sync53 --profile awsprofile --region af-south-1 --hosted-zone-id /hostedzone/Z31T92M9QYO6S8 --record-name foo.com.
```