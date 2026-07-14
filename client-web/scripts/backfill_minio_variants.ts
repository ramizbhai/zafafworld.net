import { S3Client, ListObjectsV2Command, GetObjectCommand, PutObjectCommand } from '@aws-sdk/client-s3';
import sharp from 'sharp';
import dotenv from 'dotenv';
import path from 'path';

dotenv.config({ path: path.resolve(import.meta.dir, '../.env') });

const s3 = new S3Client({
  region: 'us-east-1', // MinIO defaults to this
  endpoint: process.env.MINIO_ENDPOINT || 'http://localhost:9000',
  forcePathStyle: true,
  credentials: {
    accessKeyId: process.env.MINIO_ACCESS_KEY || 'minioadmin',
    secretAccessKey: process.env.MINIO_SECRET_KEY || 'minioadmin',
  },
});

const BUCKET = process.env.MINIO_BUCKET || 'zafafworld';

async function streamToBuffer(stream: any): Promise<Buffer> {
  const chunks = [];
  for await (const chunk of stream) {
    chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}

async function processImage(key: string, buffer: Buffer) {
  const original = sharp(buffer);
  const metadata = await original.metadata();
  const width = metadata.width || 0;
  const height = metadata.height || 0;

  const baseKey = key.replace(/\.webp$/i, '');

  console.log(`Processing variants for ${key}... (Original: ${width}x${height})`);

  // Variant 3: Medium (800px)
  let mediumBuf = buffer;
  if (width > 800 || height > 800) {
    mediumBuf = await original.clone().resize(800, 800, { fit: 'inside', withoutEnlargement: true }).webp({ quality: 80 }).toBuffer();
  }
  await uploadVariant(`${baseKey}_medium.webp`, mediumBuf);

  // Variant 4: Card (400px)
  let cardBuf = mediumBuf;
  const mediumMeta = await sharp(mediumBuf).metadata();
  if ((mediumMeta.width || 0) > 400 || (mediumMeta.height || 0) > 400) {
    cardBuf = await sharp(mediumBuf).resize(400, 400, { fit: 'inside', withoutEnlargement: true }).webp({ quality: 75 }).toBuffer();
  }
  await uploadVariant(`${baseKey}_card.webp`, cardBuf);

  // Variant 5: Thumbnail (150x150 crop)
  const thumbBuf = await sharp(cardBuf).resize(150, 150, { fit: 'cover' }).webp({ quality: 75 }).toBuffer();
  await uploadVariant(`${baseKey}_thumb.webp`, thumbBuf);
  
  console.log(`✅ Completed ${key}\n`);
}

async function uploadVariant(key: string, buffer: Buffer) {
  await s3.send(
    new PutObjectCommand({
      Bucket: BUCKET,
      Key: key,
      Body: buffer,
      ContentType: 'image/webp',
    })
  );
  console.log(`   Uploaded ${key}`);
}

async function main() {
  const execute = process.argv.includes('--execute') || process.argv.includes('--force');
  
  console.log(`Connecting to MinIO bucket: ${BUCKET}...`);
  console.log(`Execution mode: ${execute ? 'LIVE EXECUTE' : 'DRY RUN (ReadOnly)'}`);
  if (!execute) {
    console.log(`DEFAULT SAFE MODE: Performing a dry-run check. Pass --execute to run live.\n`);
  }
  
  let continuationToken: string | undefined = undefined;
  const files: string[] = [];

  do {
    const res = await s3.send(
      new ListObjectsV2Command({
        Bucket: BUCKET,
        ContinuationToken: continuationToken,
      })
    );

    if (res.Contents) {
      for (const obj of res.Contents) {
        if (obj.Key && obj.Key.endsWith('.webp')) {
          files.push(obj.Key);
        }
      }
    }
    continuationToken = res.NextContinuationToken;
  } while (continuationToken);

  console.log(`Found ${files.length} WebP files.`);

  // Filter out the variant suffixes
  const originals = files.filter(f => !f.match(/_(thumb|card|medium|large)\.webp$/));
  console.log(`Found ${originals.length} base original images to check.`);

  let processedCount = 0;
  let skippedCount = 0;

  for (const key of originals) {
    const baseKey = key.replace(/\.webp$/i, '');
    
    // Check if variants already exist in our files list
    const hasThumb = files.includes(`${baseKey}_thumb.webp`);
    const hasCard = files.includes(`${baseKey}_card.webp`);
    const hasMedium = files.includes(`${baseKey}_medium.webp`);

    if (hasThumb && hasCard && hasMedium) {
      skippedCount++;
      continue;
    }

    if (!execute) {
      console.log(`[DRY RUN] Would process variants for ${key} (missing: ${!hasMedium ? 'medium ' : ''}${!hasCard ? 'card ' : ''}${!hasThumb ? 'thumb' : ''})`);
      processedCount++;
      continue;
    }

    console.log(`Missing variants for ${key}. Downloading...`);
    try {
      const obj = await s3.send(new GetObjectCommand({ Bucket: BUCKET, Key: key }));
      if (obj.Body) {
        const buffer = await streamToBuffer(obj.Body);
        await processImage(key, buffer);
        processedCount++;
      }
    } catch (e: any) {
      console.error(`❌ Failed to process ${key}: ${e.message}`);
    }
  }

  console.log(`\n🎉 Backfill Complete!`);
  console.log(`Processed: ${processedCount}`);
  console.log(`Skipped (already had variants): ${skippedCount}`);
}

main().catch(console.error);
