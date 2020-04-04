using Xunit;

namespace TestRunner
{
    public class Structs
    {
        [Fact]
        public void CopyTupleStruct_RoundTrip()
        {
            var original = new CopyTupleStruct(1, 2);
            Assert.Equal(1, original.Element0);
            Assert.Equal(2, original.Element1);

            var result = IntegrationTests.RoundTripCopyTupleStruct(original);
            Assert.Equal(original, result);
            Assert.Equal(original.Element0, result.Element0);
            Assert.Equal(original.Element1, result.Element1);
        }

        [Fact]
        public void CopyNewtypeStruct_RoundTrip()
        {
            var original = new CopyNewtypeStruct(123);
            Assert.Equal(123, original.Element0);

            var result = IntegrationTests.RoundTripCopyNewtypeStruct(original);
            Assert.Equal(original, result);
            Assert.Equal(original.Element0, result.Element0);
        }
    }
}
