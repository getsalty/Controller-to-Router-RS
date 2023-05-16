using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using Microsoft.Extensions.Primitives;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace Test.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    public class Test2Controller : ControllerBase
    {
        private ILogger log;
        private readonly IDbConnectionFactory<testDbContext> dbFactory;
        private readonly LibraryDal libraryDal;
        private readonly ApplicationConfiguration appConfig;
        private readonly TestSessionService testSessionService;
        private readonly TestConfig testConfig;
        private readonly string testFolder;
        private readonly string testSecondFolder;

        public Test2Controller(
                ILogger<Test2Controller> log,
                IDbConnectionFactory<testDbContext> dbFactory,
                IUrlScheme urlScheme,
                IOptions<ApplicationConfiguration> appConfig,
                TestSessionService testSessionService,
                IOptions<TestConfig> testConfig)
        {
            this.log = log;
            this.dbFactory = dbFactory;
            this.appConfig = appConfig.Value;
            this.libraryDal = new LibraryDal(dbFactory, urlScheme, this.appConfig);
            this.testFolder = this.libraryDal.GetTestFolder();
            this.testSecondFolder = this.libraryDal.GetTestSecondFolder();
            this.testSessionService = testSessionService;
            this.testConfig = testConfig.Value;
        }

        [HttpPost("uploadSession")]
        public ActionResult CreateUploadSession(UploadSessionRequest request)
        {
            if (!ModelState.IsValid)
            {
                return BadRequest();
            }
            try
            {
                var result = testSessionService.CreateUploadSession(request.FileName);
                var response = new UploadSessionDto(result);
                return Ok(response);
            }
            catch(Exception ex)
            {
                log.LogError(ex, "Error creating upload");
                return Problem();
            }
        }

        [HttpGet("uploadSession/{id}")]
        public ActionResult GetUploadSession(string id)
        {
            if(!Guid.TryParse(id, out var uid))
            {
                return BadRequest();
            }

            try
            {
                var result = testSessionService.GetUploadSession(uid);
                return result == null ? NotFound() : Ok(new UploadSessionDto(result));
            }
            catch(Exception ex)
            {
                log.LogError(ex, "Error getting upload");
                return Problem();
            }
        }

        [HttpDelete("uploadSession/{id}")]
        public ActionResult DeleteUploadSession(string id)
        {
            if(!Guid.TryParse(id, out var uid))
            {
                return BadRequest();
            }

            try
            {
                testSessionService.DeleteUploadSession(uid);
                return Ok();
            }
            catch(Exception ex)
            {
                log.LogError(ex, "Error deleting upload");
                return Problem();
            }
        }
        
        [HttpPost("chunk")]
        [RequestFormLimits(ValueLengthLimit = int.MaxValue, MultipartBodyLengthLimit = uint.MaxValue)]
        public ActionResult FileChunk()
        {
            try
            {
                if (Request.Form.Files.Count != 1)
                {
                    log.LogError("Chunk message had {ActualNumFiles} but can only accept 1", Request.Form.Files.Count);
                    return BadRequest();
                }
            }
            catch (Exception)
            {
                log.LogError("Chunk message received without a form file");
                return BadRequest();
            }

            var file = Request.Form.Files[0];
            var chunkUploadIdHeader = Request.Headers["ChunkUploadSessionId"];
            var chunkNumberHeader = Request.Headers["ChunkNumber"];
            var isLastChunkHeader = Request.Headers["IsLastChunk"];

            if(!Guid.TryParse(chunkUploadIdHeader, out var uid) || !int.TryParse(chunkNumberHeader, out var chunkNumber) || !bool.TryParse(isLastChunkHeader, out var isLastChunk))
            {
                log.LogError("Chunk message missing one or more required headers");
                return BadRequest();
            }

            try
            {
                testSessionService.SaveChunk(uid, chunkNumber, file, isLastChunk);
                return Ok();
            }
            catch(Exception ex)
            {
                log.LogError(ex, "Error trying to save chunk");
                return Problem();
            }
        }

        [HttpPost("upload")]
        [RequestFormLimits(ValueLengthLimit = int.MaxValue, MultipartBodyLengthLimit = uint.MaxValue)]
        public ActionResult UploadFile(List<IFormFile> _)
        {
            try
            {
                if (Request.Form.Files.Count != 1)
                {
                    return BadRequest();
                }
            }
            catch (Exception)
            {
                return BadRequest();
            }

            var file = Request.Form.Files[0];

            if (!IsFileValidUpload(file))
            {
                return ValidationProblem("File has an invalid extension.");
            }


            try
            {                
                var filePaths = SaveFileToDirectory(file);
                
                var newFileUid = this.libraryDal.AddUploadJob(filePaths[3], this.testFolder);

                var result = new FileResult() { FilePaths = filePaths, FileUid = newFileUid };
                return Ok(result);
            }
            catch (Exception ex)
            {testFolderHttp
                return Problem(ex.Message, null, 500);
            }
        }

        /// <summary>
        /// This is a test summary
        /// </summary>
        /// <param name="file">test param</param>
        /// <exception cref="Exception"></exception>
        private string[] SaveFileToDirectory(IFormFile file)
        {
            var testFolder = this.libraryDal.GetFolders().FirstOrDefault(x => x.FolderName == "Test");
            if (testFolder == null)
            {
                throw new Exception("Test directory not found");
            }

            var testFolderUnc = testFolder.Unc;
            var testFolderHttp = testFolder.Http;
            if (testFolderUnc is null || testFolderHttp is null)
            {
                throw new Exception("Test directory's paths not found");
            }
            
            var tempFileNameWithPathUnc = Path.Combine(testFolderUnc, "temp_" + file.FileName);
            var fileNameWithPathUnc = Path.Combine(testFolderUnc, file.FileName);
            
            using (var stream = new FileStream(tempFileNameWithPathUnc, FileMode.Create))
            {
                file.CopyTo(stream);
            }

            try
            {
                using (var stream = new FileStream(fileNameWithPathUnc, FileMode.Create))
                {
                    file.CopyTo(stream);
                }
            }
            catch (Exception ex)
            {
                log.LogError(ex, "Creating file has failed");
                throw ex;
            }
            finally
            {               
                System.IO.File.Delete(tempFileNameWithPathUnc);
            }

            return new string[] { fileNameWithPathUnc, Path.Combine(testFolderHttp, file.FileName) };
        }
       
        private bool IsFileValidUpload(IFormFile file)
        {
            var fileInfo = new FileInfo(file.FileName);
            return testConfig.FileExtensions.Contains(fileInfo.Extension.ToLower());
        }
    }
}
